# Calculator Project

A simple Python calculator library that provides basic arithmetic operations.

## Project Structure

```
/workspace/
├── src/
│   ├── __init__.py
│   └── calculator.py
├── tests/
│   └── test_calculator.py
└── README.md
```

## Features

The `Calculator` class provides the following methods:

- `add(a, b)` - Addition
- `subtract(a, b)` - Subtraction
- `multiply(a, b)` - Multiplication
- `divide(a, b)` - Division (with zero-division protection)

## Installation

No external dependencies required. This project uses Python's standard library.

## Usage

### Basic Example

```python
from src.calculator import Calculator

# Create a calculator instance
calc = Calculator()

# Perform operations
result = calc.add(5, 3)        # 8
result = calc.subtract(10, 4)  # 6
result = calc.multiply(6, 7)   # 42
result = calc.divide(10, 2)    # 5.0
```

### Division by Zero Handling

The calculator gracefully handles division by zero:

```python
from src.calculator import Calculator

calc = Calculator()

try:
    result = calc.divide(10, 0)
except ValueError as e:
    print(e)  # "Cannot divide by zero"
```

### Working with Decimals

All methods support floating-point numbers:

```python
from src.calculator import Calculator

calc = Calculator()

result = calc.add(1.5, 2.3)      # 3.8
result = calc.divide(7, 2)       # 3.5
result = calc.multiply(2.5, 4)   # 10.0
```

## Running Tests

The project includes comprehensive unit tests with 10 test cases covering various scenarios.

### Using unittest (built-in)

```bash
python3 -m unittest tests/test_calculator.py -v
```

### Using pytest (if available)

```bash
python3 -m pytest tests/ -v
```

## Test Coverage

The test suite includes:

1. Addition with positive numbers
2. Addition with negative numbers
3. Subtraction with positive numbers
4. Multiplication of numbers
5. Multiplication by zero
6. Division of positive numbers
7. Division with decimal results
8. Division by zero error handling
9. Addition with floating-point numbers
10. Subtraction with floating-point numbers

## License

This is a demonstration project for educational purposes.
