# Python Calculator Project

A simple Python calculator module providing basic arithmetic operations.

## Project Structure

```
/workspace/
├── src/
│   ├── __init__.py      # Package initialization
│   └── calculator.py    # Calculator class implementation
├── tests/
│   ├── __init__.py      # Test package initialization
│   └── test_calculator.py  # Unit tests
└── README.md            # This file
```

## Features

The `Calculator` class provides the following operations:

- **Addition** (`add`): Add two numbers
- **Subtraction** (`subtract`): Subtract the second number from the first
- **Multiplication** (`multiply`): Multiply two numbers
- **Division** (`divide`): Divide the first number by the second (with zero division handling)

## Installation

No external dependencies required. Python 3.6+ recommended.

## Usage Examples

### Basic Usage

```python
from src import Calculator

# Create a calculator instance
calc = Calculator()

# Addition
result = calc.add(5, 3)
print(f"5 + 3 = {result}")  # Output: 5 + 3 = 8

# Subtraction
result = calc.subtract(10, 4)
print(f"10 - 4 = {result}")  # Output: 10 - 4 = 6

# Multiplication
result = calc.multiply(7, 6)
print(f"7 × 6 = {result}")  # Output: 7 × 6 = 42

# Division
result = calc.divide(20, 4)
print(f"20 ÷ 4 = {result}")  # Output: 20 ÷ 4 = 5.0
```

### Working with Floating-Point Numbers

```python
calc = Calculator()

# All operations support floats
print(calc.add(1.5, 2.7))       # 4.2
print(calc.multiply(3.14, 2))   # 6.28
print(calc.divide(7, 2))        # 3.5
```

### Error Handling

```python
calc = Calculator()

# Division by zero raises a ValueError
try:
    result = calc.divide(10, 0)
except ValueError as e:
    print(f"Error: {e}")  # Output: Error: Cannot divide by zero
```

## Running Tests

### Using pytest (if available)

```bash
python3 -m pytest tests/ -v
```

### Using unittest

```bash
python3 -m unittest discover -s tests -v
```

### Running tests directly

```bash
python3 tests/test_calculator.py
```

## Test Coverage

The test suite includes:

1. Addition of positive numbers
2. Addition of negative numbers
3. Addition of mixed positive/negative numbers
4. Subtraction operations
5. Multiplication operations
6. Multiplication by zero
7. Division operations
8. Division by zero error handling
9. Floating-point arithmetic

## License

This project is open source and available for educational purposes.
