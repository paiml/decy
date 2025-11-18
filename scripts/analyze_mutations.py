#!/usr/bin/env python3
"""
Mutation Testing Analysis Script
Analyzes cargo-mutants output and enforces quality thresholds
Reference: docs/specifications/improve-testing-quality-using-certeza-concepts.md

Usage:
    python scripts/analyze_mutations.py mutants.json

Targets (from decy-quality.toml):
    - Overall: ≥85% mutation score
    - decy-ownership: ≥90%
    - decy-lifetime: ≥90%
    - decy-verify: ≥85%
    - decy-parser: ≥80%
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple

# Mutation score thresholds (from decy-quality.toml)
THRESHOLDS = {
    'decy-ownership': 90.0,
    'decy-lifetime': 90.0,
    'decy-verify': 85.0,
    'decy-parser': 80.0,
    'decy-codegen': 85.0,
    'decy-analyzer': 80.0,
    'decy-hir': 80.0,
    'decy-core': 85.0,
}

OVERALL_THRESHOLD = 85.0


def load_mutants_json(filepath: str) -> Dict:
    """Load cargo-mutants JSON output"""
    with open(filepath, 'r') as f:
        return json.load(f)


def calculate_mutation_score(data: Dict) -> Tuple[float, int, int]:
    """
    Calculate mutation score from cargo-mutants data

    Returns:
        (score, caught, total)
    """
    total = 0
    caught = 0

    for outcome in data.get('outcomes', []):
        total += 1
        if outcome.get('summary') in ['caught', 'timeout']:
            caught += 1

    if total == 0:
        return 0.0, 0, 0

    score = (caught / total) * 100.0
    return score, caught, total


def calculate_per_crate_scores(data: Dict) -> Dict[str, Tuple[float, int, int]]:
    """Calculate mutation scores per crate"""
    crate_stats = {}

    for outcome in data.get('outcomes', []):
        # Extract crate name from file path
        file_path = outcome.get('scenario', {}).get('file', '')
        if 'crates/' in file_path:
            parts = file_path.split('crates/')[1].split('/')
            crate_name = parts[0] if parts else 'unknown'
        else:
            crate_name = 'unknown'

        if crate_name not in crate_stats:
            crate_stats[crate_name] = {'total': 0, 'caught': 0}

        crate_stats[crate_name]['total'] += 1
        if outcome.get('summary') in ['caught', 'timeout']:
            crate_stats[crate_name]['caught'] += 1

    # Convert to scores
    results = {}
    for crate, stats in crate_stats.items():
        if stats['total'] > 0:
            score = (stats['caught'] / stats['total']) * 100.0
            results[crate] = (score, stats['caught'], stats['total'])
        else:
            results[crate] = (0.0, 0, 0)

    return results


def analyze_surviving_mutants(data: Dict) -> List[Dict]:
    """Identify mutants that survived (tests didn't catch them)"""
    surviving = []

    for outcome in data.get('outcomes', []):
        if outcome.get('summary') not in ['caught', 'timeout']:
            surviving.append({
                'file': outcome.get('scenario', {}).get('file', 'unknown'),
                'function': outcome.get('scenario', {}).get('function', 'unknown'),
                'line': outcome.get('scenario', {}).get('line', 0),
                'summary': outcome.get('summary', 'unknown'),
            })

    return surviving


def print_report(overall_score: float, caught: int, total: int,
                 crate_scores: Dict[str, Tuple[float, int, int]],
                 surviving: List[Dict]) -> None:
    """Print formatted mutation testing report"""

    print("=" * 80)
    print("MUTATION TESTING REPORT")
    print("=" * 80)
    print()

    # Overall score
    print(f"Overall Mutation Score: {overall_score:.1f}%")
    print(f"  Caught: {caught}/{total} mutants")

    status = "✅ PASS" if overall_score >= OVERALL_THRESHOLD else "❌ FAIL"
    print(f"  Status: {status} (threshold: {OVERALL_THRESHOLD}%)")
    print()

    # Per-crate scores
    print("Per-Crate Mutation Scores:")
    print("-" * 80)

    failed_crates = []
    for crate in sorted(crate_scores.keys()):
        score, caught_count, total_count = crate_scores[crate]
        threshold = THRESHOLDS.get(crate, OVERALL_THRESHOLD)

        passed = score >= threshold
        status_icon = "✅" if passed else "❌"

        print(f"{status_icon} {crate:20} {score:6.1f}% ({caught_count:3}/{total_count:3}) "
              f"[threshold: {threshold:.0f}%]")

        if not passed:
            failed_crates.append(crate)

    print()

    # Surviving mutants (tests didn't catch)
    if surviving:
        print(f"Surviving Mutants: {len(surviving)}")
        print("-" * 80)
        for i, mutant in enumerate(surviving[:10], 1):  # Show first 10
            print(f"{i}. {mutant['file']}:{mutant['line']} in {mutant['function']}")
            print(f"   Status: {mutant['summary']}")

        if len(surviving) > 10:
            print(f"   ... and {len(surviving) - 10} more")
        print()

    # Summary
    print("=" * 80)
    if overall_score >= OVERALL_THRESHOLD and not failed_crates:
        print("✅ MUTATION TESTING PASSED")
    else:
        print("❌ MUTATION TESTING FAILED")
        if failed_crates:
            print(f"   Failed crates: {', '.join(failed_crates)}")
        if overall_score < OVERALL_THRESHOLD:
            print(f"   Overall score {overall_score:.1f}% < {OVERALL_THRESHOLD}%")
    print("=" * 80)


def main():
    if len(sys.argv) < 2:
        print("Usage: python scripts/analyze_mutations.py mutants.json")
        sys.exit(1)

    mutants_file = sys.argv[1]

    if not Path(mutants_file).exists():
        print(f"Error: File not found: {mutants_file}")
        sys.exit(1)

    # Load and analyze data
    data = load_mutants_json(mutants_file)
    overall_score, caught, total = calculate_mutation_score(data)
    crate_scores = calculate_per_crate_scores(data)
    surviving = analyze_surviving_mutants(data)

    # Print report
    print_report(overall_score, caught, total, crate_scores, surviving)

    # Exit with error if thresholds not met
    failed_crates = [
        crate for crate, (score, _, _) in crate_scores.items()
        if score < THRESHOLDS.get(crate, OVERALL_THRESHOLD)
    ]

    if overall_score < OVERALL_THRESHOLD or failed_crates:
        sys.exit(1)

    sys.exit(0)


if __name__ == '__main__':
    main()
