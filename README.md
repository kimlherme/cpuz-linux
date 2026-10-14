# CPU-Z-like Benchmark for Linux

A C++17 command-line benchmark that reproduces part of the workload format used
by the CPU test in CPU-Z 19.01. It measures how many pixels per second the
software-rendered workload can process.

This is an independent scalar implementation. The full details of the original
algorithm were not recovered, so this project is useful for experimentation
and comparing runs of this implementation. Its score is not an official CPU-Z
score and should not be compared directly with the score shown by CPU-Z.

## Contents

- [Requirements](#requirements)
- [Build](#build)
- [Run](#run)
- [Interactive terminal UI](#interactive-terminal-ui)
- [Options](#options)
- [Understanding the output](#understanding-the-output)
- [How the benchmark works](#how-the-benchmark-works)
- [Limitations and comparing results](#limitations-and-comparing-results)
- [Troubleshooting](#troubleshooting)

## Requirements

- Linux
- A C++ compiler with C++17 support, such as GCC or Clang
- `make` (optional; you can also compile with a direct command)
- POSIX thread support, usually provided by the Linux standard library

On Debian or Ubuntu, install the build tools with:

```sh
sudo apt update
sudo apt install build-essential
```

## Build

From the repository root:

```sh
make
```

This creates the `cpu-z-bench-linux` executable. To compile without `make`:

```sh
g++ -O3 -std=c++17 -Wall -Wextra -Wpedantic -ffp-contract=off -pthread \
  cpu-z-bench-linux.cpp -o cpu-z-bench-linux
```

Remove the generated executable with:

```sh
make clean
```

## Run

With no arguments, the program runs a 10-second single-threaded benchmark:

```sh
./cpu-z-bench-linux
```

Examples with an explicit mode and duration:

```sh
# Single-threaded for 10 seconds
./cpu-z-bench-linux --mode single --duration-ms 10000

# Multi-threaded using the CPUs allowed to this process
./cpu-z-bench-linux --mode multi --duration-ms 10000 --threads auto

# Multi-threaded with 4 workers
./cpu-z-bench-linux --mode multi --duration-ms 10000 --threads 4

# Run multi-threaded, then single-threaded, for 5 seconds each
./cpu-z-bench-linux --mode both --duration-ms 5000 --threads 4
```

Show the built-in help with:

```sh
./cpu-z-bench-linux --help
```

## Interactive terminal UI

Choose settings and start runs from a terminal menu:

```sh
./cpu-z-bench-linux --tui
```

In the menu, `m` cycles through single, multi, and both modes; `d` sets the
duration in milliseconds; `w` sets the worker count or `auto`; `r` starts a
run; `h` shows the controls; and `q` exits. The menu returns after each run.
The selected duration applies separately to each mode, and each run performs a
150 ms warmup before the timer starts.

You can also provide initial settings when opening the menu:

```sh
./cpu-z-bench-linux --tui --mode both --duration-ms 3000 --threads auto
```

The interface uses standard terminal input and output, requires no extra
dependencies, and needs an interactive terminal session.

## Options

| Option | Values | Default | Description |
|---|---|---|---|
| `--mode` | `single`, `multi`, or `both` | `single` | Select single-threaded, multi-threaded, or both runs. |
| `--duration-ms` | `1` to `86400000` | `10000` | Approximate measured duration of each run, in milliseconds. |
| `--threads` | `auto` or `1` to `1024` | `auto` | Number of workers for multi-threaded runs. Ignored in `single` mode. |
| `--tui` | — | disabled | Open the interactive terminal menu. |
| `--help`, `-h` | — | — | Show the available options. |

### Modes

- `single`: processes frames with one thread. This is the default and is most
  useful for observing the performance of one logical CPU.
- `multi`: splits image rows among the workers and measures parallel processing.
  It does not also run the single-threaded test.
- `both`: runs the multi-threaded test first, waits 10 ms, then runs the
  single-threaded test. The selected duration applies to each run separately.

In multi-threaded mode, `--threads auto` uses the number of CPUs in the
process's affinity mask when Linux provides it. If affinity cannot be queried,
the program uses the count reported by the standard library. `auto` does not
detect CPU quota limits that are not reflected in the affinity mask.

## Understanding the output

Each run reports elapsed time, the number of fully processed pixels, throughput
in pixels per second, and a score on the approximate CPU-Z 19.01 scale:

```text
Single-thread:
  CPU: Intel(R) ...
  reported clock (/proc/cpuinfo): 3400.0-4100.0 MHz
  elapsed: ... s
  pixels: ...
  pixels/s: ...
  19.01-scale score (pixels/s / 10000): ...
```

The metrics are calculated as follows:

```text
pixels/s = processed pixels / elapsed time in seconds
scale score = pixels/s / 10,000
```

Each image position is counted once. The count is not multiplied by the four
RGBA channels, the noise layers, or the number of internal operations.

Results vary with the processor, clock speed, temperature, system load, virtual
machine or container limits, and selected duration. The score is rounded to one
decimal place, so check `pixels/s` when comparing small differences.

The CPU model comes from the `model name` field in `/proc/cpuinfo`. The reported
clock is the range of available `cpu MHz` values from that file, read after each
run. It is an informational sample, can vary between CPUs, and may be
unavailable in virtual machines.

## How the benchmark works

Before each measured run, the program performs a 150 ms warmup using the same
mode and worker count. Warmup work is excluded from the reported pixel count
and elapsed time. This helps the processor move out of its idle state,
especially during short runs.

The program creates a deterministic table of 4096 values before measurement.
It then processes 1024 × 1024 pixel images. For each pixel, it computes two
simplex-style 2D noise compositions: one with 10 layers and one with 7. The
results are converted into four color channels (RGBA) and written to a buffer.

In single-threaded mode, one thread processes the entire image. In
multi-threaded mode, workers receive separate row ranges and synchronize between
frames. Thread and buffer setup happens before the timed interval.

The program stops when the selected duration ends and counts only completed
rows. The actual duration may therefore vary slightly around the selected
value because the timer is checked while processing rows.

## Limitations and comparing results

This version was reconstructed from static analysis of the CPU-Z 19.01 test
path. The analysis did not recover every detail of the table, hash, composition
formulas, or color conversion, so these parts are approximations. This
implementation also uses a scalar kernel and does not include CPU-Z's SSE,
AVX2, or AVX-512 SIMD paths.

For these reasons, `19.01-scale score` identifies only the scale used to
convert throughput; it does not make the results equivalent to the official
score. A score of `163.8`, for example, means this implementation measured
about 1,638,000 pixels/s. It does not mean the official CPU-Z score on the same
processor would be 163.8.

To compare changes to this program, use the same machine, mode, duration, and
thread count. When possible, run it more than once under similar conditions
and compare `pixels/s`. Close heavy background tasks, and record whether the
benchmark ran on physical hardware, in a virtual machine, or in a container. In
virtualized environments, visible CPUs and host-enforced quotas can limit the
results.

## Troubleshooting

### `g++: command not found` or `make: command not found`

Install the compiler and build tools for your distribution. On Debian or Ubuntu,
run `sudo apt install build-essential`.

### `Unknown option` or invalid value

Use `--help` to check the syntax. `--duration-ms` accepts values from 1 to
86400000; `--threads` accepts `auto` or values from 1 to 1024.

### Results differ between runs

Differences can come from CPU frequency and temperature changes, system load,
affinity, virtual machine quotas, or resource contention. Repeat the benchmark
under similar conditions before interpreting small differences.

## Project status

This is an experimental, independent Linux benchmark. It is not an official
implementation and is not affiliated with CPUID, the maker of CPU-Z. This
project is licensed under the GNU General Public License, version 2
(GPL-2.0-only); see [LICENSE](LICENSE).
