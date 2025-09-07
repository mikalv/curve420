
# python_ed420/tests/test_curve.py

import pytest
from curve import FieldElement, EdwardsPoint, P, A_EDWARDS, D_EDWARDS, G, L_ORDER, H_COFACTOR

# Test FieldElement
def test_field_element_arithmetic():
    fe1 = FieldElement(10)
    fe2 = FieldElement(5)

    assert (fe1 + fe2).value == 15 % P
    assert (fe1 - fe2).value == 5 % P
    assert (fe1 * fe2).value == 50 % P
    assert (fe1 / fe2).value == (10 * pow(5, P - 2, P)) % P
    assert (-fe1).value == (P - 10) % P
    assert fe1 == 10
    assert fe1 != 11

def test_field_element_inverse():
    fe = FieldElement(7)
    assert (fe * fe.inverse()).value == 1

def test_field_element_is_square_and_sqrt():
    # Test a known quadratic residue
    qr = FieldElement(pow(2, 2, P)) # 4
    assert qr.is_square()
    assert qr.sqrt().value == 2 or qr.sqrt().value == (P - 2)

    # Test a known non-quadratic residue (example, might need to adjust for specific P)
    # For P = 2^420 - 335, we need to find a non-QR
    # A simple way is to check if a random number is not a square
    # For testing, we can pick a small non-QR if P % 4 == 3, then -1 is non-QR
    if P % 4 == 3:
        non_qr = FieldElement(P - 1) # -1 mod P
        assert not non_qr.is_square()
        with pytest.raises(ValueError):
            non_qr.sqrt()

# Test EdwardsPoint
def test_edwards_point_on_curve():
    # Test the canonical base point
    assert G.is_on_curve()

    # Test a point not on the curve (by changing x slightly)
    invalid_x = FieldElement(G.x.value + 1)
    invalid_point = EdwardsPoint(invalid_x, G.y)
    assert not invalid_point.is_on_curve()

def test_edwards_point_addition():
    # Test G + G (doubling)
    G_doubled = G + G
    assert G_doubled.is_on_curve()

    # Test G + Identity
    identity = EdwardsPoint(FieldElement(0), FieldElement(1))
    assert (G + identity) == G
    assert (identity + G) == G

def test_edwards_point_scalar_multiplication():
    # Test G * 2
    G_times_2 = G * 2
    assert G_times_2.is_on_curve()
    assert G_times_2 == (G + G)

    # Test G * 0
    identity = EdwardsPoint(FieldElement(0), FieldElement(1))
    assert (G * 0) == identity

    # Test G * L_ORDER (should be identity)
    l_times_g = G * L_ORDER
    identity = EdwardsPoint(FieldElement(0), FieldElement(1))
    assert l_times_g == identity

    # Test G * H_COFACTOR (should not be identity)
    h_times_g = G * H_COFACTOR
    assert h_times_g != identity

