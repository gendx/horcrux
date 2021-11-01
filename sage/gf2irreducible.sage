# Sage script to find irreducible polynomials over GF(2).
#
# This only tries polynomials with an odd number of terms. Indeed, other
# polynomials are divisible by `x + 1`, because 1 is a root.
#
# For each number of terms (3, 5 or 7), the script uses a brute-force approach
# to find the smallest irreducible polynomial, according to lexicographical
# order.

k = GF(2, 't')
R = PolynomialRing(k, 'u')
u = R.gen()

def smallest_single_irreducible(n):
    for i in range(1, n):
        f = u^n + u^i + 1
        factors = list(f.factor())
        if len(factors) == 1 and factors[0][1] == 1:
            return f
    return None

def smallest_triple_irreducible(n):
    count = 0
    for i in range(3, 32):
        for j in range(2, i):
            for k in range(1, j):
                count += 1
                f = u^n + u^i + u^j + u^k + 1
                factors = list(f.factor())
                if len(factors) == 1 and factors[0][1] == 1:
                    return (count, f)
    return None

def smallest_quintet_irreducible(n):
    count = 0
    for i in range(5, 32):
        for j in range(4, i):
            for k in range(3, j):
                for l in range(2, k):
                    for m in range(1, l):
                        count += 1
                        f = u^n + u^i + u^j + u^k + u^l + u^m + 1
                        factors = list(f.factor())
                        if len(factors) == 1 and factors[0][1] == 1:
                            return (count, f)
    return None

def print_irreducible_polynomials(n):
    print(n)
    f1 = smallest_single_irreducible(n)
    print("\t", f1)
    f3 = smallest_triple_irreducible(n)
    print("\t", f3)
    f5 = smallest_quintet_irreducible(n)
    print("\t", f5)

for n in range(8, 257):
    print_irreducible_polynomials(n)

for i in range(1, 33):
    print_irreducible_polynomials(i * 64)
