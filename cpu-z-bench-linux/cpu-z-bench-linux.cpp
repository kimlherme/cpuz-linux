#include <algorithm>
#include <array>
#include <atomic>
#include <charconv>
#include <chrono>
#include <cmath>
#include <condition_variable>
#include <cstdint>
#include <cstdlib>
#include <iomanip>
#include <iostream>
#include <mutex>
#include <sched.h>
#include <string>
#include <thread>
#include <unistd.h>
#include <vector>

namespace {

constexpr int kWidth = 1024;
constexpr int kHeight = 1024;
constexpr int kTableSize = 4096;
constexpr int kTableMask = kTableSize - 1;
using Clock = std::chrono::steady_clock;

struct Options {
    std::uint64_t duration_ms = 10'000;
    unsigned threads = 0;  // 0 means all CPUs allowed to this process.
    std::string mode = "single";
    bool tui = false;
};

struct Result {
    std::uint64_t pixels = 0;
    double elapsed_seconds = 0.0;
};

class NoiseTable {
public:
    NoiseTable() {
        // The table is deterministic in the analyzed CPU-Z 19.01 path. This
        // follows its sin/hash/truncation shape and stays outside the timer.
        constexpr double kTau = 6.283185005187988;
        constexpr double kInvTableSize = 1.0 / 4096.0;
        constexpr double kHashScale = 43758.546875;
        for (int i = 0; i < kTableSize; ++i) {
            const double phase = static_cast<double>(i) * kInvTableSize * kTau;
            const double hash = std::sin(phase) * kHashScale;
            const double fractional = hash - std::trunc(hash);
            values_[i] = static_cast<float>(fractional * 2.0 - 1.0);
        }
    }

    float primitive(float x, float y) const {
        constexpr float kF2 = 0.3660254180431366f;
        constexpr float kG2 = 0.21132487058639526f;
        constexpr float kInvSqrt3 = 0.5773502588272095f;

        const float skew = (x + y) * kF2;
        const int i = fast_floor(x + skew);
        const int j = fast_floor(y + skew);
        const float unskew = static_cast<float>(i + j) * kG2;
        const float x0 = x - (static_cast<float>(i) - unskew);
        const float y0 = y - (static_cast<float>(j) - unskew);
        const int i1 = x0 > y0 ? 1 : 0;
        const int j1 = x0 > y0 ? 0 : 1;
        const float x1 = x0 - static_cast<float>(i1) + kG2;
        const float y1 = y0 - static_cast<float>(j1) + kG2;
        const float x2 = x0 - 1.0f + 2.0f * kG2;
        const float y2 = y0 - 1.0f + 2.0f * kG2;

        return (corner(i, j, x0, y0) +
                corner(i + i1, j + j1, x1, y1) +
                corner(i + 1, j + 1, x2, y2)) * kInvSqrt3;
    }

    float composition_a(int pixel_x, int pixel_y, std::uint32_t frame) const {
        static constexpr std::array<float, 10> kAbsoluteWeights = {
            0.8f, 0.56f, 0.392f, 0.2744f, 0.19208f,
            0.134456f, 0.09411919f, 0.06588344f, 0.04611840f, 0.03228288f};
        static constexpr std::array<float, 10> kSignedWeights = {
            0.7f, 0.42000002f, 0.25200003f, 0.15120003f, 0.09072002f,
            0.05443202f, 0.03265921f, 0.01959553f, 0.01175732f, 0.00705439f};
        constexpr float kStartScale = 1.1f / 1024.0f;

        const float frame_offset = static_cast<float>(frame);
        float x = (static_cast<float>(pixel_x) + frame_offset) * kStartScale;
        float y = (static_cast<float>(pixel_y) + frame_offset) * kStartScale;
        float absolute_sum = 0.0f;
        float signed_sum = 0.0f;
        for (std::size_t layer = 0; layer < kAbsoluteWeights.size(); ++layer) {
            const float value = primitive(x, y) * 40.0f;
            absolute_sum += std::fabs(value) * kAbsoluteWeights[layer];
            signed_sum += value * kSignedWeights[layer];
            rotate(x, y);
        }

        // The static review confirms a product and a 0.2 term, but not their
        // exact placement. This composition remains an explicit approximation.
        return absolute_sum * signed_sum + 0.2f;
    }

    float composition_b(int pixel_x, int pixel_y, std::uint32_t frame) const {
        static constexpr std::array<float, 7> kWeights = {
            0.4f, 0.14400001f, 0.08640001f, 0.05184001f,
            0.03110401f, 0.01866240f, 0.01119744f};
        constexpr float kStartScale = 2.2f / 1024.0f;

        const float frame_offset = static_cast<float>(frame);
        float x = (static_cast<float>(pixel_x) + frame_offset) * kStartScale;
        float y = (static_cast<float>(pixel_y) + frame_offset) * kStartScale;
        float weighted_sum = 0.0f;
        for (std::size_t layer = 0; layer < kWeights.size(); ++layer) {
            const float value = primitive(x, y) * 40.0f;
            weighted_sum += value * kWeights[layer];
            rotate(x, y);
        }
        return weighted_sum + std::fabs(weighted_sum);
    }

    static std::uint8_t color(float value) {
        if (value < 0.0f) value = 0.0f;
        if (value > 1.0f) value = 1.0f;
        return static_cast<std::uint8_t>(value * 255.0f);
    }

private:
    static int fast_floor(float value) {
        const int truncated = static_cast<int>(value);
        return value < static_cast<float>(truncated) ? truncated - 1 : truncated;
    }

    static void rotate(float& x, float& y) {
        const float old_x = x;
        const float old_y = y;
        x = 1.6f * old_x - 1.2f * old_y;
        y = 1.2f * old_x + 1.6f * old_y;
    }

    float table_helper(float entry) const {
        constexpr double kInvTau = 0.15915494309189533577;
        const double turns = static_cast<double>(entry) * kInvTau;
        const double fractional = turns - std::trunc(turns);
        const int index = static_cast<int>(fractional * -4096.0);
        return values_[static_cast<unsigned>(index) & kTableMask];
    }

    float corner(int lattice_x, int lattice_y, float x, float y) const {
        float attenuation = 0.5f - x * x - y * y;
        if (attenuation < 0.0f) {
            return 0.0f;
        }
        const float hash = static_cast<float>(lattice_x) * 127.1f +
                           static_cast<float>(lattice_y) * 311.7f;
        const float gx = table_helper(hash);
        const float gy = table_helper(hash + 1.57079632679f);
        attenuation *= attenuation;
        const float gradient = gx * x + gy * y;
        return attenuation * attenuation * gradient;
    }

    std::array<float, kTableSize> values_{};
};

std::uint64_t render_rows(const NoiseTable& noise,
                         int first_row,
                         int row_count,
                         std::uint32_t frame,
                         std::vector<std::uint8_t>& pixels,
                         Clock::time_point deadline) {
    std::uint64_t completed_pixels = 0;
    for (int local_y = 0; local_y < row_count; ++local_y) {
        if (Clock::now() >= deadline) {
            break;
        }

        const int y = first_row + local_y;
        std::size_t offset = static_cast<std::size_t>(local_y) * kWidth * 4;
        for (int x = 0; x < kWidth; ++x) {
            const float a = noise.composition_a(x, y, frame);
            const float b = noise.composition_b(x, y, frame);
            pixels[offset++] = NoiseTable::color(a + 0.5f);
            pixels[offset++] = NoiseTable::color(b + 0.5f);
            pixels[offset++] = NoiseTable::color(a + b + 0.2f);
            pixels[offset++] = 0xff;
        }

#if defined(__GNUC__) || defined(__clang__)
        // Keep each rendered row observable to the optimizer without adding a
        // runtime checksum to the measured pixel loop.
        asm volatile("" : : "g"(pixels.data()) : "memory");
#endif
        completed_pixels += kWidth;
    }
    return completed_pixels;
}

class FrameBarrier {
public:
    explicit FrameBarrier(unsigned participants) : participants_(participants) {}

    void arrive_and_wait() {
        std::unique_lock<std::mutex> lock(mutex_);
        const unsigned generation = generation_;
        if (++arrived_ == participants_) {
            arrived_ = 0;
            ++generation_;
            condition_.notify_all();
        } else {
            condition_.wait(lock, [&] { return generation_ != generation; });
        }
    }

private:
    const unsigned participants_;
    unsigned arrived_ = 0;
    unsigned generation_ = 0;
    std::mutex mutex_;
    std::condition_variable condition_;
};

Result run_single(const NoiseTable& noise, std::chrono::milliseconds duration) {
    std::vector<std::uint8_t> pixels(static_cast<std::size_t>(kWidth) * kHeight * 4);
    const auto start = Clock::now();
    const auto deadline = start + duration;
    std::uint64_t completed = 0;
    std::uint32_t frame = 0;

    while (Clock::now() < deadline) {
        const std::uint64_t batch = render_rows(noise, 0, kHeight, frame, pixels, deadline);
        completed += batch;
        if (batch < static_cast<std::uint64_t>(kWidth) * kHeight) {
            break;
        }
        ++frame;
    }

    const auto end = Clock::now();
    return {completed, std::chrono::duration<double>(end - start).count()};
}

Result run_multi(const NoiseTable& noise,
                 unsigned thread_count,
                 std::chrono::milliseconds duration) {
    struct Worker {
        int first_row = 0;
        int row_count = 0;
        std::vector<std::uint8_t> pixels;
        std::uint64_t completed = 0;
        std::uint32_t frame = 0;
    };

    std::vector<Worker> workers(thread_count);
    for (unsigned i = 0; i < thread_count; ++i) {
        Worker& worker = workers[i];
        worker.first_row = static_cast<int>((static_cast<std::uint64_t>(kHeight) * i) / thread_count);
        const int end_row = static_cast<int>((static_cast<std::uint64_t>(kHeight) * (i + 1)) / thread_count);
        worker.row_count = end_row - worker.first_row;
        worker.pixels.resize(static_cast<std::size_t>(worker.row_count) * kWidth * 4);
    }

    std::atomic<unsigned> ready{0};
    std::atomic<bool> start_workers{false};
    std::atomic<bool> stop_workers{false};
    FrameBarrier frame_barrier(thread_count);
    Clock::time_point deadline;
    std::vector<std::thread> threads;
    threads.reserve(thread_count);

    for (unsigned i = 0; i < thread_count; ++i) {
        threads.emplace_back([&, i] {
            Worker& worker = workers[i];
            ready.fetch_add(1, std::memory_order_release);
            while (!start_workers.load(std::memory_order_acquire)) {
                std::this_thread::yield();
            }

            while (true) {
                if (Clock::now() >= deadline) {
                    stop_workers.store(true, std::memory_order_relaxed);
                }
                if (!stop_workers.load(std::memory_order_relaxed)) {
                    const std::uint64_t batch = render_rows(
                        noise, worker.first_row, worker.row_count, worker.frame,
                        worker.pixels, deadline);
                    worker.completed += batch;
                    if (batch < static_cast<std::uint64_t>(worker.row_count) * kWidth) {
                        stop_workers.store(true, std::memory_order_relaxed);
                    } else {
                        ++worker.frame;
                    }
                }
                frame_barrier.arrive_and_wait();
                if (stop_workers.load(std::memory_order_relaxed)) break;
            }
        });
    }

    while (ready.load(std::memory_order_acquire) != thread_count) {
        std::this_thread::yield();
    }
    const auto start = Clock::now();
    deadline = start + duration;
    start_workers.store(true, std::memory_order_release);
    for (std::thread& thread : threads) {
        thread.join();
    }
    const auto end = Clock::now();

    std::uint64_t completed = 0;
    for (const Worker& worker : workers) {
        completed += worker.completed;
    }
    return {completed, std::chrono::duration<double>(end - start).count()};
}

unsigned allowed_cpu_count() {
    cpu_set_t set;
    CPU_ZERO(&set);
    if (sched_getaffinity(0, sizeof(set), &set) == 0) {
        const int count = CPU_COUNT(&set);
        if (count > 0) {
            return static_cast<unsigned>(std::min(count, kHeight));
        }
    }

    const unsigned fallback = std::thread::hardware_concurrency();
    return fallback == 0 ? 1 : std::min(fallback, static_cast<unsigned>(kHeight));
}

bool parse_unsigned(const std::string& text, std::uint64_t& value) {
    if (text.empty()) {
        return false;
    }
    const char* begin = text.data();
    const char* end = begin + text.size();
    const auto parsed = std::from_chars(begin, end, value);
    return parsed.ec == std::errc{} && parsed.ptr == end;
}

void print_usage(const char* program) {
    std::cout
        << "Usage: " << program << " [options]\n"
        << "  --mode single|multi|both  Run mode (default: single)\n"
        << "  --duration-ms N  Duration of each run (default: 10000)\n"
        << "  --threads N|auto Workers in the multi-thread run (default: auto)\n"
        << "  --tui            Open the interactive terminal menu\n"
        << "  --help           Show this help\n";
}

bool parse_options(int argc, char** argv, Options& options) {
    options.threads = allowed_cpu_count();
    for (int i = 1; i < argc; ++i) {
        const std::string arg = argv[i];
        if (arg == "--help" || arg == "-h") {
            print_usage(argv[0]);
            std::exit(0);
        }
        if (arg == "--tui") {
            options.tui = true;
            continue;
        }
        if (arg == "--mode") {
            if (i + 1 >= argc) {
                std::cerr << "Missing value for --mode\n";
                return false;
            }
            options.mode = argv[++i];
            if (options.mode != "single" && options.mode != "multi" && options.mode != "both") {
                std::cerr << "--mode must be single, multi, or both\n";
                return false;
            }
            continue;
        }
        if (arg == "--duration-ms" || arg == "--threads") {
            if (i + 1 >= argc) {
                std::cerr << "Missing value for " << arg << "\n";
                return false;
            }
            const std::string value = argv[++i];
            if (arg == "--threads" && value == "auto") {
                options.threads = allowed_cpu_count();
                continue;
            }

            std::uint64_t parsed = 0;
            if (!parse_unsigned(value, parsed)) {
                std::cerr << "Invalid non-negative integer for " << arg << ": " << value << "\n";
                return false;
            }
            if (arg == "--duration-ms") {
                if (parsed == 0 || parsed > 86'400'000) {
                    std::cerr << "--duration-ms must be between 1 and 86400000\n";
                    return false;
                }
                options.duration_ms = parsed;
            } else if (arg == "--threads") {
                if (parsed == 0 || parsed > kHeight) {
                    std::cerr << "--threads must be between 1 and 1024, or 'auto'\n";
                    return false;
                }
                options.threads = static_cast<unsigned>(parsed);
            }
            continue;
        }

        std::cerr << "Unknown option: " << arg << "\n";
        return false;
    }
    return true;
}

void print_result(const char* label, const Result& result);

void print_tui_menu(const Options& options) {
    std::cout << "\n==============================================\n"
              << " CPU-Z Linux benchmark - interactive tester\n"
              << "==============================================\n"
              << "Mode: " << options.mode << "\n"
              << "Duration: " << options.duration_ms << " ms\n"
              << "Workers: " << options.threads << " (multi/both)\n\n"
              << "  m  Change mode (single / multi / both)\n"
              << "  d  Change duration in milliseconds\n"
              << "  w  Change worker count (or use auto)\n"
              << "  r  Run with these settings\n"
              << "  h  Show controls\n"
              << "  q  Quit\n"
              << "> " << std::flush;
}

int run_tui(Options options, const NoiseTable& noise) {
    if (!isatty(STDIN_FILENO) || !isatty(STDOUT_FILENO)) {
        std::cerr << "--tui requires an interactive terminal\n";
        return 2;
    }

    std::string command;
    while (true) {
        print_tui_menu(options);
        if (!std::getline(std::cin, command)) {
            std::cout << "\n";
            return 0;
        }

        if (command == "q" || command == "Q" || command == "quit") {
            return 0;
        }
        if (command == "h" || command == "H" || command == "help") {
            std::cout << "Enter m, d, w, or r at the prompt.\n"
                      << "Duration is per run; 'both' runs multi, waits 10 ms, then single.\n"
                      << "A duration of 1000 ms is useful for quick comparisons.\n";
            continue;
        }
        if (command == "m" || command == "M") {
            if (options.mode == "single") {
                options.mode = "multi";
            } else if (options.mode == "multi") {
                options.mode = "both";
            } else {
                options.mode = "single";
            }
            continue;
        }
        if (command == "d" || command == "D") {
            std::cout << "Duration in milliseconds (1-86400000): " << std::flush;
            std::string value;
            if (!std::getline(std::cin, value)) {
                std::cout << "\n";
                return 0;
            }
            std::uint64_t parsed = 0;
            if (!parse_unsigned(value, parsed) || parsed == 0 || parsed > 86'400'000) {
                std::cout << "Invalid duration; settings unchanged.\n";
            } else {
                options.duration_ms = parsed;
            }
            continue;
        }
        if (command == "w" || command == "W") {
            std::cout << "Workers (1-1024 or auto): " << std::flush;
            std::string value;
            if (!std::getline(std::cin, value)) {
                std::cout << "\n";
                return 0;
            }
            if (value == "auto") {
                options.threads = allowed_cpu_count();
            } else {
                std::uint64_t parsed = 0;
                if (!parse_unsigned(value, parsed) || parsed == 0 || parsed > kHeight) {
                    std::cout << "Invalid worker count; settings unchanged.\n";
                } else {
                    options.threads = static_cast<unsigned>(parsed);
                }
            }
            continue;
        }
        if (command == "r" || command == "R" || command == "run") {
            const auto duration = std::chrono::milliseconds(options.duration_ms);
            std::cout << "\nRunning " << options.mode << " benchmark; please wait.\n";
            if (options.mode != "single") {
                const Result multi = run_multi(noise, options.threads, duration);
                print_result("Multi-thread", multi);
            }
            if (options.mode == "both") {
                std::this_thread::sleep_for(std::chrono::milliseconds(10));
            }
            if (options.mode != "multi") {
                const Result single = run_single(noise, duration);
                print_result("Single-thread", single);
            }
            continue;
        }

        std::cout << "Unknown command. Enter h for controls.\n";
    }
}

void print_result(const char* label, const Result& result) {
    const double pixels_per_second = result.elapsed_seconds > 0.0
        ? static_cast<double>(result.pixels) / result.elapsed_seconds
        : 0.0;
    const double score = pixels_per_second / 10'000.0;
    std::cout << label << ":\n"
              << "  elapsed: " << result.elapsed_seconds << " s\n"
              << "  pixels: " << result.pixels << "\n"
              << "  pixels/s: " << pixels_per_second << "\n"
              << "  19.01-scale score (pixels/s / 10000): "
              << std::fixed << std::setprecision(1) << score << std::defaultfloat << "\n";
}

}  // namespace

int main(int argc, char** argv) {
    Options options;
    if (!parse_options(argc, argv, options)) {
        print_usage(argv[0]);
        return 2;
    }

    if (options.tui) {
        const NoiseTable noise;
        return run_tui(options, noise);
    }

    std::cout << "CPU-Z-like Linux benchmark (independent scalar implementation)\n"
              << "  mode: " << options.mode << "\n"
              << "  workload: 1024x1024 RGBA pixels; 10+7 layered 2D simplex-style noise\n"
              << "  score scale: 19.01-style pixels/s / 10000, 64-bit pixel counter\n"
              << "  duration per run: " << options.duration_ms << " ms\n";
    if (options.mode != "single") {
        std::cout << "  multi-thread workers: " << options.threads << "\n";
    }
    std::cout
              << "  note: reconstructed scalar kernel; hash/composition/color details remain approximate\n";

    const NoiseTable noise;
    const auto duration = std::chrono::milliseconds(options.duration_ms);

    if (options.mode != "single") {
        const Result multi = run_multi(noise, options.threads, duration);
        print_result("Multi-thread", multi);
    }
    if (options.mode == "both") {
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
    if (options.mode != "multi") {
        const Result single = run_single(noise, duration);
        print_result("Single-thread", single);
    }
    return 0;
}
