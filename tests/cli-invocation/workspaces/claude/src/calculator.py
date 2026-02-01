"""
Calculator module providing basic arithmetic operations.
"""


class Calculator:
    """
    A simple calculator class that performs basic arithmetic operations.

    This class provides methods for addition, subtraction, multiplication,
    and division with proper error handling.
    """

    def add(self, a, b):
        """
        Add two numbers together.

        Args:
            a (int or float): First number
            b (int or float): Second number

        Returns:
            int or float: Sum of a and b
        """
        return a + b

    def subtract(self, a, b):
        """
        Subtract the second number from the first.

        Args:
            a (int or float): Number to subtract from
            b (int or float): Number to subtract

        Returns:
            int or float: Difference of a and b
        """
        return a - b

    def multiply(self, a, b):
        """
        Multiply two numbers together.

        Args:
            a (int or float): First number
            b (int or float): Second number

        Returns:
            int or float: Product of a and b
        """
        return a * b

    def divide(self, a, b):
        """
        Divide the first number by the second.

        Args:
            a (int or float): Numerator
            b (int or float): Denominator

        Returns:
            float: Quotient of a and b

        Raises:
            ValueError: If b is zero (division by zero)
        """
        if b == 0:
            raise ValueError("Cannot divide by zero")
        return a / b
