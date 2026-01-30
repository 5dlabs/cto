"""
Calculator module providing basic arithmetic operations.
"""


class Calculator:
    """
    A simple calculator class that performs basic arithmetic operations.

    Methods:
        add(a, b): Returns the sum of two numbers
        subtract(a, b): Returns the difference of two numbers
        multiply(a, b): Returns the product of two numbers
        divide(a, b): Returns the quotient of two numbers
    """

    def add(self, a, b):
        """
        Add two numbers together.

        Args:
            a (float): The first number
            b (float): The second number

        Returns:
            float: The sum of a and b
        """
        return a + b

    def subtract(self, a, b):
        """
        Subtract the second number from the first.

        Args:
            a (float): The number to subtract from
            b (float): The number to subtract

        Returns:
            float: The difference of a and b
        """
        return a - b

    def multiply(self, a, b):
        """
        Multiply two numbers together.

        Args:
            a (float): The first number
            b (float): The second number

        Returns:
            float: The product of a and b
        """
        return a * b

    def divide(self, a, b):
        """
        Divide the first number by the second.

        Args:
            a (float): The dividend
            b (float): The divisor

        Returns:
            float: The quotient of a and b

        Raises:
            ValueError: If b is zero (division by zero)
        """
        if b == 0:
            raise ValueError("Cannot divide by zero")
        return a / b
