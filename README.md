# Curve420

Curve420 is a cryptographic elliptic curve providing approximately 210 bits of security. It is defined over the prime field `p = 2^420 - 335`. The curve can be represented in two forms: Twisted Edwards and Montgomery.

The parameters for Curve420 were generated deterministically using SHAKE-128, based on the "nothing up my sleeve" principle. The chosen parameters correspond to index 5769 from the search process.

## Formal Definition

### Prime Field

The curve is defined over the prime field F_p where:
`p = 2^420 - 335`
`p = 2707685248164858261307045101702230179137145581421695874189921465443966120903931272499975005961073806735733604454495675614232241`

### Twisted Edwards Curve

The Twisted Edwards form of the curve is given by the equation:
`a*x^2 + y^2 = 1 + d*x^2*y^2`

**Parameters:**
- `a = 763975519699500577645754547835125169481986463482154078046572648671788968290548038674290307302429817161505744408446033521089604`
- `d = 763975519699500577645754547835125169481986463482154078046572648671788968290548038674290307302429817161505744408446033521089600`

**Base Point (G):**
- `x = 2554519045303036994902077297242990796196199161457630080356703041833906288977089421513471756737913123939108844302244613830350009`
- `y = 1554004282195909523747673681974014268960308454695342458183393593582942692590987497223833263666951454840260505456918987028153736`

### Montgomery Curve

The Montgomery form of the curve is given by the equation:
`y^2 = x^3 + A*x^2 + x`

**Parameters:**
- `A = 763975519699500577645754547835125169481986463482154078046572648671788968290548038674290307302429817161505744408446033521089602`

**Base Point (u-coordinate):**
- `u = 1887066872174968132246224128199266266323489104588603923691363826518154582291788366769852665419756146257203683605002692187211605`

### Group Properties

- **Prime Subgroup Order (l):**
  `l = 338460656020607282663380637712778772392143197677711984273740183501508577674026655281164768623743539442603492250355597371718719`
  (a 418-bit prime number)

- **Cofactor (h):**
  `h = 8`

- **Curve Order (N):**
  `N = h * l = 2707685248164858261307045101702230179137145581421695874189921468012068621392213242249318148989948315540827938002844778973749752`

## Alternative Generators

While any point `k*G` (where `k` is an integer coprime to the subgroup order `l`) is a valid generator of the prime-order subgroup, for interoperability and standardization, it is crucial to use the canonical base point `G` specified above. Other generators, while mathematically valid, may not be interoperable with implementations adhering to this standard unless explicitly agreed upon by all parties.

## Security Considerations

*   **j-invariant:** The j-invariant of the curve is not 0 or 1728. The exact value needs to be calculated and documented.
*   **Twist Security:** The order of the quadratic twist of the curve needs to be analyzed to ensure it is secure.

This document is a work in progress. The parameters are extracted from the reference implementation in Rust.
Further analysis is required to confirm all security properties.
