import json
import math
import sys
from pathlib import Path


class SoakAnalysis:
    RSS_BUDGET = 64 * 1024 * 1024
    DESCRIPTOR_RANGE = 8
    DRIFT = 1.10
    FAULT_GRACE_MS = {"kill": 32000.0, "timeout": 2001.0, "invalid_runtime": 62000.0}

    def __init__(self, path, short=False):
        self.short = short
        self.path = Path(path)
        self.rows = []
        self.complete = None
        for line in self.path.read_text().splitlines():
            record = json.loads(line)
            if "complete" in record:
                self.complete = record
            else:
                self.rows.append(record)

    @staticmethod
    def rank(values, p):
        ordered = sorted(values)
        return ordered[math.ceil(len(ordered) * p) - 1] if ordered else None

    def warm(self):
        return self.rows[2:]

    def criteria(self):
        normal = [row for row in self.rows if row["fault"] == "normal"]
        faults = [row for row in self.rows if row["fault"] != "normal"]
        slow_faults = [row["cycle"] for row in faults
                       if row.get("fault_ms", row["elapsed_ms"]) > self.FAULT_GRACE_MS[row["fault"]]]
        leftovers = [row["cycle"] for row in self.rows if row["worker_survived"] or row["worker_zombie"] or row["private_entries"] != 0]
        warm = self.warm()
        rss = [row["rss_bytes"] for row in warm]
        descriptors = [row["open_descriptors"] for row in warm]
        hour = 3600 * 1000.0
        clock, first_hour, last_hour = 0.0, [], []
        stamps = []
        for row in self.rows:
            clock += row["elapsed_ms"]
            stamps.append((clock, row))
        total = clock
        for at, row in stamps:
            if row["fault"] == "normal" and "first_ms" in row:
                if at <= hour:
                    first_hour.append(row["first_ms"])
                if at >= total - hour:
                    last_hour.append(row["first_ms"])
        first_p95, last_p95 = self.rank(first_hour, 0.95), self.rank(last_hour, 0.95)
        return {
            "R1.1_normal_bits_equal": all(row["ok"] for row in normal),
            "R1.1_failed_normal_cycles": [row["cycle"] for row in normal if not row["ok"]],
            "R1.2_faults_typed": all(row["ok"] for row in faults),
            "R1.2_failed_fault_cycles": [row["cycle"] for row in faults if not row["ok"]],
            "R1.2_faults_within_bound": not slow_faults,
            "R1.2_slow_fault_cycles": slow_faults,
            "R1.3_no_survivors_zombies_or_copies": not leftovers,
            "R1.3_leftover_cycles": leftovers,
            "R1.4_post_warm_rss_range_mib": (max(rss) - min(rss)) / 1048576 if rss else None,
            "R1.4_rss_within_64_mib": bool(rss) and max(rss) - min(rss) <= self.RSS_BUDGET,
            "R1.5_descriptor_range": (max(descriptors) - min(descriptors)) if descriptors else None,
            "R1.5_descriptors_within_8": bool(descriptors) and max(descriptors) - min(descriptors) <= self.DESCRIPTOR_RANGE,
            "R1.6_first_hour_p95_ms": first_p95,
            "R1.6_last_hour_p95_ms": last_p95,
            "R1.6_drift_within_1_10": first_p95 is not None and last_p95 is not None and last_p95 <= self.DRIFT * first_p95,
            "wall_clock_hours": total / hour,
            "R1_duration_at_least_3h": total >= 3 * hour,
            "cycles": len(self.rows),
            "cycles_by_fault": {name: sum(row["fault"] == name for row in self.rows) for name in ("normal", "kill", "timeout", "invalid_runtime")},
            "completed_normally": self.complete is not None,
        }

    def run(self):
        result = self.criteria()
        excluded = {"R1_duration_at_least_3h", "R1.6_drift_within_1_10"} if self.short else set()
        if self.short:
            result["R3_short_at_least_50_cycles"] = result["cycles"] >= 50
        gates = [key for key, value in result.items()
                 if key.startswith(("R1", "R3")) and isinstance(value, bool) and key not in excluded]
        result["mode"] = "short (R3: R1.1-R1.5 and at least 50 cycles)" if self.short else "full R1"
        result["R1_passed"] = all(result[key] for key in gates)
        print(json.dumps(result, indent=2))
        return 0 if result["R1_passed"] else 1


if __name__ == "__main__":
    sys.exit(SoakAnalysis(sys.argv[1], short="--short" in sys.argv[2:]).run())
