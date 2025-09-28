import time

def count_to_million():
    count = 0
    for _ in range(1_000_000):
        count += 1
    return count

def main():
    start_time = time.perf_counter()
    result = count_to_million()
    end_time = time.perf_counter()

    duration_micros = (end_time - start_time) * 1_000_000
    print(f"Python counted to {result} in {duration_micros:.0f} microseconds")

if __name__ == "__main__":
    main()
