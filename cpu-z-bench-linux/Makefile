CXX ?= g++
CXXFLAGS ?= -O3 -std=c++17 -Wall -Wextra -Wpedantic -ffp-contract=off
LDFLAGS ?=
LDLIBS ?= -pthread

TARGET := cpu-z-bench-linux
SOURCE := cpu-z-bench-linux.cpp

.PHONY: all clean

all: $(TARGET)

$(TARGET): $(SOURCE)
	$(CXX) $(CXXFLAGS) $(LDFLAGS) -pthread $< -o $@ $(LDLIBS)

clean:
	rm -f $(TARGET)
