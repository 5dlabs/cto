"""
Unit tests for the Calculator class.

This module contains comprehensive tests for all Calculator operations
including edge cases like division by zero.
"""

import unittest
import sys
import os

# Add the src directory to the path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from calculator import Calculator


class TestCalculator(unittest.TestCase):
    """Test cases for the Calculator class."""

    def setUp(self):
        """Set up a Calculator instance for each test."""
        self.calc = Calculator()

    def test_add_positive_numbers(self):
        """Test addition of two positive numbers."""
        result = self.calc.add(2, 3)
        self.assertEqual(result, 5)

    def test_add_negative_numbers(self):
        """Test addition of two negative numbers."""
        result = self.calc.add(-5, -3)
        self.assertEqual(result, -8)

    def test_add_mixed_numbers(self):
        """Test addition of positive and negative numbers."""
        result = self.calc.add(10, -4)
        self.assertEqual(result, 6)

    def test_subtract_positive_numbers(self):
        """Test subtraction of two positive numbers."""
        result = self.calc.subtract(10, 4)
        self.assertEqual(result, 6)

    def test_subtract_resulting_negative(self):
        """Test subtraction resulting in a negative number."""
        result = self.calc.subtract(3, 7)
        self.assertEqual(result, -4)

    def test_multiply_positive_numbers(self):
        """Test multiplication of two positive numbers."""
        result = self.calc.multiply(3, 4)
        self.assertEqual(result, 12)

    def test_multiply_by_zero(self):
        """Test multiplication by zero."""
        result = self.calc.multiply(5, 0)
        self.assertEqual(result, 0)

    def test_multiply_negative_numbers(self):
        """Test multiplication of negative numbers."""
        result = self.calc.multiply(-3, -4)
        self.assertEqual(result, 12)

    def test_divide_positive_numbers(self):
        """Test division of two positive numbers."""
        result = self.calc.divide(10, 2)
        self.assertEqual(result, 5.0)

    def test_divide_with_remainder(self):
        """Test division that results in a decimal."""
        result = self.calc.divide(7, 2)
        self.assertEqual(result, 3.5)

    def test_divide_by_zero_raises_error(self):
        """Test that division by zero raises ValueError."""
        with self.assertRaises(ValueError) as context:
            self.calc.divide(10, 0)
        self.assertEqual(str(context.exception), "Cannot divide by zero")

    def test_add_floats(self):
        """Test addition of floating-point numbers."""
        result = self.calc.add(1.5, 2.5)
        self.assertEqual(result, 4.0)

    def test_subtract_floats(self):
        """Test subtraction of floating-point numbers."""
        result = self.calc.subtract(5.5, 2.5)
        self.assertEqual(result, 3.0)


if __name__ == '__main__':
    unittest.main()
