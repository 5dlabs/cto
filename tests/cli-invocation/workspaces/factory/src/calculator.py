"""
Calculator module providing basic arithmetic operations.

This module contains a Calculator class with methods for addition,
subtraction, multiplication, and division operations.
"""


class Calculator:
    """
    A simple calculator class that performs basic arithmetic operations.

    This class provides methods to add, subtract, multiply, and divide numbers.
    All methods support both integer and floating-point numbers.

    Example:
        >>> calc = Calculator()
        >>> calc.add(2, 3)
        5
        >>> calc.divide(10, 2)
        5.0
    """

    def add(self, a: float, b: float) -> float:
        """
        Add two numbers together.

        Args:
            a: The first number.
            b: The second number.

        Returns:
            The sum of a and b.

        Example:
            >>> calc = Calculator()
            >>> calc.add(5, 3)
            8
        """
        return a + b

    def subtract(self, a: float, b: float) -> float:
        """
        Subtract the second number from the first.

        Args:
            a: The number to subtract from.
            b: The number to subtract.

        Returns:
            The difference of a and b.

        Example:
            >>> calc = Calculator()
            >>> calc.subtract(10, 4)
            6
        """
        return a - b

    def multiply(self, a: float, b: float) -> float:
        """
        Multiply two numbers together.

        Args:
            a: The first number.
            b: The second number.

        Returns:
            The product of a and b.

        Example:
            >>> calc = Calculator()
            >>> calc.multiply(3, 4)
            12
        """
        return a * b

    def divide(self, a: float, b: float) -> float:
        """
        Divide the first number by the second.

        Args:
            a: The dividend (number to be divided).
            b: The divisor (number to divide by).

        Returns:
            The quotient of a divided by b.

        Raises:
            ValueError: If b is zero (division by zero).

        Example:
            >>> calc = Calculator()
            >>> calc.divide(10, 2)
            5.0
        """
        if b == 0:
            raise ValueError("Cannot divide by zero")
        return a / b
