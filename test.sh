#!/bin/bash
# dms-awww Testing & Benchmarking Script
# This script runs unit tests, integration tests, and benchmarks

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASS=0
FAIL=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASS++)) || true
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAIL++)) || true
}

info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

section() {
    echo ""
    echo "======================================"
    echo "$1"
    echo "======================================"
}

# Parse arguments
RUN_UNIT=false
RUN_INTEGRATION=false
RUN_BENCH=false
RUN_ALL=true
VERBOSE=false

for arg in "$@"; do
    case $arg in
        --unit)
            RUN_UNIT=true
            RUN_ALL=false
            ;;
        --integration)
            RUN_INTEGRATION=true
            RUN_ALL=false
            ;;
        --bench)
            RUN_BENCH=true
            RUN_ALL=false
            ;;
        --all)
            RUN_ALL=true
            ;;
        --verbose|-v)
            VERBOSE=true
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --unit         Run unit tests only"
            echo "  --integration  Run integration tests only"
            echo "  --bench        Run benchmarks only"
            echo "  --all          Run all tests and benchmarks (default)"
            echo "  --verbose, -v  Enable verbose output"
            echo "  --help, -h     Show this help message"
            exit 0
            ;;
    esac
done

section "DMS-AWWW Testing Suite"

# Run unit tests
if [ "$RUN_UNIT" = true ] || [ "$RUN_ALL" = true ]; then
    section "Unit Tests"
    info "Running cargo test --lib..."

    if cargo test --lib $([ "$VERBOSE" = true ] && echo "-- --nocapture" || echo "-- --quiet"); then
        pass "Unit tests passed"
    else
        fail "Unit tests failed"
    fi
fi

# Run integration tests
if [ "$RUN_INTEGRATION" = true ] || [ "$RUN_ALL" = true ]; then
    section "Integration Tests"
    info "Running cargo test --test-threads=1..."

    if cargo test --test-threads=1 $([ "$VERBOSE" = true ] && echo "-- --nocapture" || echo "-- --quiet"); then
        pass "Integration tests passed"
    else
        fail "Integration tests failed"
    fi
fi

# Run benchmarks
if [ "$RUN_BENCH" = true ] || [ "$RUN_ALL" = true ]; then
    section "Benchmarks"
    info "Running cargo bench (this may take a while)..."

    # Check if criterion is available
    if cargo bench -- --test 2>&1 | grep -q "Success"; then
        info "Config benchmarks..."
        cargo bench --bench config_bench $([ "$VERBOSE" = false ] && echo "-- --quiet" || echo "") 2>&1 | grep -E "(Testing|Time)" | tail -5

        info "DMS benchmarks..."
        cargo bench --bench dms_bench $([ "$VERBOSE" = false ] && echo "-- --quiet" || echo "") 2>&1 | grep -E "(Testing|Time)" | tail -5

        info "Comparison benchmarks..."
        cargo bench --bench comparison $([ "$VERBOSE" = false ] && echo "-- --quiet" || echo "") 2>&1 | grep -E "(Testing|Time)" | tail -5

        pass "Benchmarks completed"
    else
        warn "Benchmarks skipped (criterion not available in release mode)"
    fi
fi

# Summary
section "Test Summary"
echo -e "${GREEN}Passed: $PASS${NC}"
if [ $FAIL -gt 0 ]; then
    echo -e "${RED}Failed: $FAIL${NC}"
fi

if [ $FAIL -eq 0 ]; then
    echo ""
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}Some tests failed. Run with --verbose for details.${NC}"
    exit 1
fi
