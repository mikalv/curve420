
# python_ed420/src/ed420/curve.py

import math

# Field prime
P = (1 << 420) - 335

class FieldElement:
    def __init__(self, value):
        if not isinstance(value, int):
            raise TypeError("FieldElement value must be an integer.")
        self.value = value % P

    def __add__(self, other):
        if isinstance(other, FieldElement):
            return FieldElement(self.value + other.value)
        return FieldElement(self.value + other)

    def __sub__(self, other):
        if isinstance(other, FieldElement):
            return FieldElement(self.value - other.value)
        return FieldElement(self.value - other)

    def __mul__(self, other):
        if isinstance(other, FieldElement):
            return FieldElement(self.value * other.value)
        return FieldElement(self.value * other)

    def __truediv__(self, other):
        if isinstance(other, FieldElement):
            return self * other.inverse()
        return self * FieldElement(other).inverse()

    def __pow__(self, power):
        if not isinstance(power, int) or power < 0:
            raise ValueError("Power must be a non-negative integer.")
        return FieldElement(pow(self.value, power, P))

    def __eq__(self, other):
        if isinstance(other, FieldElement):
            return self.value == other.value
        return self.value == other

    def __neg__(self):
        return FieldElement(-self.value)

    def __repr__(self):
        return f"FieldElement({self.value})"

    def __str__(self):
        return str(self.value)

    def inverse(self):
        return self.__pow__(P - 2) # Fermat's Little Theorem

    def is_square(self):
        if self.value == 0:
            return True
        return self.__pow__((P - 1) // 2) == 1

    def sqrt(self):
        if not self.is_square():
            raise ValueError("Value is not a quadratic residue.")
        # Tonelli-Shanks algorithm (simplified for P % 4 == 3)
        # For P % 4 == 3, sqrt(n) = n^((P+1)/4) mod P
        if P % 4 == 3:
            return self.__pow__((P + 1) // 4)
        # More general Tonelli-Shanks would be needed for other primes
        raise NotImplementedError("sqrt for P % 4 != 3 not implemented.")

# Curve parameters (from curve420_canonical.json)
A_EDWARDS = FieldElement(763975519699500577645754547835125169481986463482154078046572648671788968290548038674290307302429817161505744408446033521089604)
D_EDWARDS = FieldElement(763975519699500577645754547835125169481986463482154078046572648671788968290548038674290307302429817161505744408446033521089600)

# Base point G (Edwards)
G_X = FieldElement(2554519045303036994902077297242990796196199161457630080356703041833906288977089421513471756737913123939108844302244613830350009)
G_Y = FieldElement(1554004282195909523747673681974014268960308454695342458183393593582942692590987497223833263666951454840260505456918987028153736)

# Group order L (prime subgroup order)
L_ORDER = 338460656020607282663380637712778772392143197677711984273740183501508577674026655281164768623743539442603492250355597371718719

# Cofactor H
H_COFACTOR = 8

class EdwardsPoint:
    def __init__(self, x, y):
        if not isinstance(x, FieldElement) or not isinstance(y, FieldElement):
            raise TypeError("EdwardsPoint coordinates must be FieldElement instances.")
        self.x = x
        self.y = y

    def __eq__(self, other):
        if not isinstance(other, EdwardsPoint):
            return NotImplemented
        return self.x == other.x and self.y == other.y

    def __repr__(self):
        return f"EdwardsPoint(x={self.x.value}, y={self.y.value})"

    def is_on_curve(self):
        lhs = A_EDWARDS * self.x**2 + self.y**2
        rhs = FieldElement(1) + D_EDWARDS * self.x**2 * self.y**2
        return lhs == rhs

    def __add__(self, other):
        if not isinstance(other, EdwardsPoint):
            raise TypeError("Can only add EdwardsPoint to another EdwardsPoint.")

        # Identity element (0, 1)
        if self.x == 0 and self.y == 1:
            return other
        if other.x == 0 and other.y == 1:
            return self

        x1, y1 = self.x, self.y
        x2, y2 = other.x, other.y

        x3_num = x1 * y2 + y1 * x2
        x3_den = FieldElement(1) + D_EDWARDS * x1 * x2 * y1 * y2

        y3_num = y1 * y2 - A_EDWARDS * x1 * x2
        y3_den = FieldElement(1) - D_EDWARDS * x1 * x2 * y1 * y2

        if x3_den == 0 or y3_den == 0:
            # This would indicate point at infinity or other special cases
            # For Edwards curves, (0,1) is the identity.
            # If a denominator is zero, it implies a special case like adding a point to its negative
            # or adding the identity.
            # For now, we'll return the identity if denominators are zero, assuming valid inputs.
            return EdwardsPoint(FieldElement(0), FieldElement(1))

        x3 = x3_num / x3_den
        y3 = y3_num / y3_den
        return EdwardsPoint(x3, y3)

    def __mul__(self, scalar):
        if not isinstance(scalar, int) or scalar < 0:
            raise ValueError("Scalar must be a non-negative integer.")

        result = EdwardsPoint(FieldElement(0), FieldElement(1)) # Identity element
        addend = self

        while scalar > 0:
            if scalar & 1: # If scalar is odd
                result += addend
            addend += addend # Double the addend
            scalar >>= 1 # Divide scalar by 2

        return result

# Canonical Base Point
G = EdwardsPoint(G_X, G_Y)

