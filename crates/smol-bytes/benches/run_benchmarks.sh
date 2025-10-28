#!/usr/bin/env bash
#
# Convenience script for running SmolBytes benchmarks
#

set -e

BENCH_NAME="clone"
CRITERION_ARGS=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --baseline)
            BASELINE_NAME="$2"
            CRITERION_ARGS="--save-baseline $BASELINE_NAME"
            shift 2
            ;;
        --compare)
            BASELINE_NAME="$2"
            CRITERION_ARGS="--baseline $BASELINE_NAME"
            shift 2
            ;;
        --quick)
            CRITERION_ARGS="--quick"
            shift
            ;;
        --filter)
            FILTER="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "OPTIONS:"
            echo "  --baseline NAME    Save results as baseline NAME"
            echo "  --compare NAME     Compare results against baseline NAME"
            echo "  --quick           Run with reduced sample size (faster)"
            echo "  --filter PATTERN  Only run benchmarks matching PATTERN"
            echo "  --help            Show this help"
            echo ""
            echo "EXAMPLES:"
            echo "  $0                                  # Run all benchmarks"
            echo "  $0 --quick                          # Quick run"
            echo "  $0 --baseline main                  # Save as 'main' baseline"
            echo "  $0 --compare main                   # Compare against 'main'"
            echo "  $0 --filter inline_only             # Only inline benchmarks"
            echo "  $0 --filter 'clone/32 bytes'        # Only 32-byte benchmarks"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Build command
CMD="cargo bench --bench $BENCH_NAME"

if [ -n "$FILTER" ]; then
    CMD="$CMD -- $FILTER"
fi

if [ -n "$CRITERION_ARGS" ]; then
    CMD="$CMD $CRITERION_ARGS"
fi

echo "Running: $CMD"
echo ""

eval "$CMD"

echo ""
echo "✓ Benchmarks complete!"
echo ""
echo "View results: open target/criterion/report/index.html"
