# Calculator Project

A simple Python calculator library that provides basic arithmetic operations with proper error handling.

## Project Structure

```
/workspace/
├── src/
│   ├── __init__.py
│   └── calculator.py      # Calculator class implementation
├── tests/
│   └── test_calculator.py # Unit tests
└── README.md              # This file
```

## Features

- Addition
- Subtraction
- Multiplication
- Division (with zero-division protection)
- Comprehensive unit tests
- Full docstring documentation

## Installation

No external dependencies required. This project uses only Python standard library.

## Usage Examples

### Basic Usage

```python
from src.calculator import Calculator

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
print(f"7 * 6 = {result}")  # Output: 7 * 6 = 42

# Division
result = calc.divide(20, 4)
print(f"20 / 4 = {result}")  # Output: 20 / 4 = 5.0
```

### Error Handling

The calculator properly handles division by zero:

```python
from src.calculator import Calculator

calc = Calculator()

try:
    result = calc.divide(10, 0)
except ValueError as e:
    print(f"Error: {e}")  # Output: Error: Cannot divide by zero
```

### Working with Decimals

All methods support both integer and floating-point numbers:

```python
from src.calculator import Calculator

calc = Calculator()

result = calc.add(2.5, 3.7)
print(f"2.5 + 3.7 = {result}")  # Output: 2.5 + 3.7 = 6.2

result = calc.divide(7, 2)
print(f"7 / 2 = {result}")  # Output: 7 / 2 = 3.5
```

## Running Tests

### Using unittest (built-in)

```bash
python3 -m unittest tests/test_calculator.py -v
```

### Using pytest (if installed)

```bash
python3 -m pytest tests/ -v
```

### Run tests from the test file directly

```bash
cd /workspace
python3 tests/test_calculator.py
```

## Test Coverage

The project includes 12 comprehensive test cases covering:

- Addition with positive and negative numbers
- Subtraction with various scenarios
- Multiplication including edge cases (zero, negatives)
- Division with decimals and error handling
- Floating-point number operations
- Division by zero error handling

## API Reference

### Calculator Class

#### `add(a, b)`
Returns the sum of two numbers.

**Parameters:**
- `a` (int or float): First number
- `b` (int or float): Second number

**Returns:** int or float

#### `subtract(a, b)`
Returns the difference between two numbers.

**Parameters:**
- `a` (int or float): Number to subtract from
- `b` (int or float): Number to subtract

**Returns:** int or float

#### `multiply(a, b)`
Returns the product of two numbers.

**Parameters:**
- `a` (int or float): First number
- `b` (int or float): Second number

**Returns:** int or float

#### `divide(a, b)`
Returns the quotient of two numbers.

**Parameters:**
- `a` (int or float): Numerator
- `b` (int or float): Denominator

**Returns:** float

**Raises:** `ValueError` if b is zero

## License

This is a demonstration project created for educational purposes.
