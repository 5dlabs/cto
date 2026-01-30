"""
Unit tests for the Calculator class.
"""

import unittest
import sys
import os

# Add the parent directory to the path to import the src module
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from src.calculator import Calculator


class TestCalculator(unittest.TestCase):
    """Test cases for the Calculator class."""

    def setUp(self):
        """Set up a Calculator instance before each test."""
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

    def test_multiply_numbers(self):
        """Test multiplication of two numbers."""
        result = self.calc.multiply(6, 7)
        self.assertEqual(result, 42)

    def test_multiply_by_zero(self):
        """Test multiplication by zero."""
        result = self.calc.multiply(5, 0)
        self.assertEqual(result, 0)

    def test_divide_positive_numbers(self):
        """Test division of two positive numbers."""
        result = self.calc.divide(10, 2)
        self.assertEqual(result, 5)

    def test_divide_with_decimal_result(self):
        """Test division that results in a decimal."""
        result = self.calc.divide(7, 2)
        self.assertEqual(result, 3.5)

    def test_divide_by_zero(self):
        """Test that division by zero raises ValueError."""
        with self.assertRaises(ValueError) as context:
            self.calc.divide(10, 0)
        self.assertEqual(str(context.exception), "Cannot divide by zero")

    def test_add_floats(self):
        """Test addition with floating point numbers."""
        result = self.calc.add(1.5, 2.3)
        self.assertAlmostEqual(result, 3.8, places=10)

    def test_subtract_floats(self):
        """Test subtraction with floating point numbers."""
        result = self.calc.subtract(5.5, 2.2)
        self.assertAlmostEqual(result, 3.3, places=10)


if __name__ == '__main__':
    unittest.main()
