"""
Unit tests for the Calculator class.
"""

import unittest
import sys
import os

# Add parent directory to path to import calculator module
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from src.calculator import Calculator


class TestCalculator(unittest.TestCase):
    """Test cases for the Calculator class."""

    def setUp(self):
        """Set up test fixtures before each test method."""
        self.calc = Calculator()

    def test_add_positive_numbers(self):
        """Test addition of two positive numbers."""
        result = self.calc.add(5, 3)
        self.assertEqual(result, 8)

    def test_add_negative_numbers(self):
        """Test addition with negative numbers."""
        result = self.calc.add(-5, -3)
        self.assertEqual(result, -8)

    def test_subtract_positive_numbers(self):
        """Test subtraction of two positive numbers."""
        result = self.calc.subtract(10, 4)
        self.assertEqual(result, 6)

    def test_subtract_negative_result(self):
        """Test subtraction resulting in negative number."""
        result = self.calc.subtract(3, 10)
        self.assertEqual(result, -7)

    def test_multiply_positive_numbers(self):
        """Test multiplication of two positive numbers."""
        result = self.calc.multiply(4, 5)
        self.assertEqual(result, 20)

    def test_multiply_by_zero(self):
        """Test multiplication by zero."""
        result = self.calc.multiply(100, 0)
        self.assertEqual(result, 0)

    def test_multiply_negative_numbers(self):
        """Test multiplication with negative numbers."""
        result = self.calc.multiply(-3, -4)
        self.assertEqual(result, 12)

    def test_divide_positive_numbers(self):
        """Test division of two positive numbers."""
        result = self.calc.divide(10, 2)
        self.assertEqual(result, 5.0)

    def test_divide_with_decimal_result(self):
        """Test division resulting in decimal."""
        result = self.calc.divide(7, 2)
        self.assertAlmostEqual(result, 3.5)

    def test_divide_by_zero_raises_error(self):
        """Test that division by zero raises ValueError."""
        with self.assertRaises(ValueError) as context:
            self.calc.divide(10, 0)
        self.assertEqual(str(context.exception), "Cannot divide by zero")

    def test_add_float_numbers(self):
        """Test addition with floating point numbers."""
        result = self.calc.add(2.5, 3.7)
        self.assertAlmostEqual(result, 6.2)

    def test_divide_negative_numbers(self):
        """Test division with negative numbers."""
        result = self.calc.divide(-10, 2)
        self.assertEqual(result, -5.0)


if __name__ == '__main__':
    unittest.main()
