#!/usr/bin/env bash
# bench-history.sh — Run benchmarks and save results to docs/benchmarks/
set -euo pipefail

BENCH_DIR="docs/benchmarks"
TIMESTAMP=$(date -u +"%Y%m%dT%H%M%SZ")
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
OUTFILE="${BENCH_DIR}/${TIMESTAMP}-${COMMIT}.txt"

mkdir -p "$BENCH_DIR"

echo "=== Ark Benchmark Run ==="
echo "Date:   $(date -u)"
echo "Commit: ${COMMIT}"
echo "========================="
echo ""

cargo bench 2>&1 | tee "$OUTFILE"

echo ""
echo "Results saved to: ${OUTFILE}"
