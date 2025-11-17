import time
import psutil
import requests
import json
import sys


# ----------------------------------------------------
# Memory helper
# ----------------------------------------------------
def mem_mb(pid):
    if not pid:
        return 0.0
    try:
        p = psutil.Process(pid)
        return p.memory_info().rss / (1024 * 1024)
    except:
        return 0.0


# ----------------------------------------------------
# Single-request measurement (CRUD)
# ----------------------------------------------------
def measure(method, url, payload=None, pid=None, session=None):
    s = session or requests.Session()

    t0_wall = time.time()
    t0_cpu = time.perf_counter()

    try:
        resp = s.request(
            method,
            url,
            data=payload,
            timeout=20,
            allow_redirects=(method.upper() == "GET")
        )
    except Exception as e:
        return {"error": str(e), "status": None}

    t1_cpu = time.perf_counter()
    t1_wall = time.time()

    mem_now = mem_mb(pid)

    return {
        "latency_ms": (t1_wall - t0_wall) * 1000,
        "exec_ms": (t1_cpu - t0_cpu) * 1000,
        "memory_mb": mem_now,
        "status": resp.status_code
    }


# ----------------------------------------------------
# Bulk Create
# ----------------------------------------------------
def bulk_create(base_url, count=100, pid=None, session=None):
    s = session or requests.Session()

    items = [{"title": f"bulk-{i}"} for i in range(count)]

    t0_wall = time.time()
    t0_cpu = time.perf_counter()

    try:
        resp = s.post(f"{base_url}/bulk_create", json=items, timeout=40)
        created_count = resp.json().get("created_count", count)
    except:
        created_count = 0

    t1_cpu = time.perf_counter()
    t1_wall = time.time()
    mem_now = mem_mb(pid)

    return {
        "latency_ms": (t1_wall - t0_wall) * 1000,
        "exec_ms": (t1_cpu - t0_cpu) * 1000,
        "memory_mb": mem_now,
        "created_count": created_count
    }


# ----------------------------------------------------
# Parse IDs from HTML
# ----------------------------------------------------
def parse_ids(html):
    import re
    ids = set()

    for m in re.finditer(r"/edit/(\d+)", html):
        ids.add(int(m.group(1)))

    for m in re.finditer(r"<td>\s*(\d+)\s*</td>", html):
        ids.add(int(m.group(1)))

    return sorted(ids)


# ----------------------------------------------------
# Bulk Update / Delete
# ----------------------------------------------------
def bulk_update(base_url, ids, pid=None, session=None):
    s = session or requests.Session()
    payload = [{"id": i, "title": f"updated-{i}", "completed": True} for i in ids]

    t0_wall = time.time()
    t0_cpu = time.perf_counter()

    try:
        resp = s.put(f"{base_url}/bulk_update", json=payload, timeout=40)
        updated = resp.json().get("updated", len(ids))
    except:
        updated = 0

    t1_cpu = time.perf_counter()
    t1_wall = time.time()
    mem_now = mem_mb(pid)

    return {
        "latency_ms": (t1_wall - t0_wall) * 1000,
        "exec_ms": (t1_cpu - t0_cpu) * 1000,
        "memory_mb": mem_now,
        "updated_count": updated,
    }


def bulk_delete(base_url, ids, pid=None, session=None):
    s = session or requests.Session()

    t0_wall = time.time()
    t0_cpu = time.perf_counter()

    try:
        resp = s.request("DELETE", f"{base_url}/bulk_delete", json=ids, timeout=30)
        deleted = resp.json().get("deleted", len(ids))
    except:
        deleted = 0

    t1_cpu = time.perf_counter()
    t1_wall = time.time()
    mem_now = mem_mb(pid)

    return {
        "latency_ms": (t1_wall - t0_wall) * 1000,
        "exec_ms": (t1_cpu - t0_cpu) * 1000,
        "memory_mb": mem_now,
        "deleted_count": deleted,
    }


# ----------------------------------------------------
# Run Complete Flow (CRUD + Bulk)
# ----------------------------------------------------
def run_all(name, base_url, pid=None, bulk_count=50):
    s = requests.Session()

    out = {}

    # CRUD
    out["CREATE"] = measure("POST", f"{base_url}/create", payload={"title": "test"}, pid=pid, session=s)
    out["READ"] = measure("GET", f"{base_url}/", pid=pid, session=s)

    # Seed one item for update/delete
    try:
        r = s.get(f"{base_url}/", timeout=10)
        ids = parse_ids(r.text)
    except:
        ids = []

    if ids:
        tid = ids[-1]
        out["UPDATE"] = measure("POST", f"{base_url}/edit/{tid}", payload={"title": "upd"}, pid=pid, session=s)
        out["DELETE"] = measure("POST", f"{base_url}/delete/{tid}", pid=pid, session=s)
    else:
        out["UPDATE"] = {"error": "no id"}
        out["DELETE"] = {"error": "no id"}

    # Bulk
    out["BULK_CREATE"] = bulk_create(base_url, count=bulk_count, pid=pid, session=s)

    # Extract new IDs
    try:
        r = s.get(f"{base_url}/", timeout=10)
        all_ids = parse_ids(r.text)
        bulk_ids = all_ids[-bulk_count:]
    except:
        bulk_ids = []

    out["BULK_READ"] = measure("GET", f"{base_url}/", pid=pid, session=s)
    out["BULK_UPDATE"] = bulk_update(base_url, ids=bulk_ids, pid=pid, session=s)
    out["BULK_DELETE"] = bulk_delete(base_url, ids=bulk_ids, pid=pid, session=s)

    return out


# ----------------------------------------------------
# Main
# ----------------------------------------------------
if __name__ == "__main__":
    base = "http://127.0.0.1:8080"
    pid = 8812         # supply PID for memory sampling
    bulk_n = 50

    try:
        result = run_all("RUST", base, pid=pid, bulk_count=bulk_n)
    except Exception as e:
        result = {"error": str(e)}

    print(json.dumps(result, indent=2))
