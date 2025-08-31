import time

def fib(n):
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    n = 20
    start_time = time.perf_counter()
    result = fib(n)
    end_time = time.perf_counter()

    duration_micros = (end_time - start_time) * 1_000_000
    print(f"Result: {result}")
    print(f"Calculating fib({n}) in Python took: {duration_micros:.0f} microseconds")

if __name__ == "__main__":
    main()
