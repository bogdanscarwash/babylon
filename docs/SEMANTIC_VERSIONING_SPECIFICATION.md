## Semantic Versioning Specification (SemVer)

The key words _"MUST"_, _"MUST NOT"_, _"REQUIRED"_, _"SHALL"_, _"SHALL NOT"_, _"SHOULD"_, _"SHOULD NOT"_, _"RECOMMENDED"_, _"MAY"_, and _"OPTIONAL"_ in this document are to be interpreted as described in RFC 2119.

- Software using Semantic Versioning **MUST** declare a public API. This API could be declared in the code itself or exist strictly in documentation. However it is done, it **SHOULD** be precise and comprehensive.

- A normal version number **MUST** take the form X.Y.Z where X, Y, and Z are non-negative integers, and **MUST NOT** contain leading zeroes. X is the major version, Y is the minor version, and Z is the patch version. Each element **MUST** increase numerically. For instance: 1.9.0 -> 1.10.0 -> 1.11.0.

- Once a versioned package has been released, the contents of that version **MUST NOT** be modified. Any modifications **MUST** be released as a new version.

- Major version zero (0.y.z) is for initial development. Anything **MAY** change at any time. The public API **SHOULD NOT** be considered stable.

- Version 1.0.0 defines the public API. The way in which the version number is incremented after this release is dependent on this public API and how it changes.

- Patch version Z (x.y.Z | x > 0) **MUST** be incremented if only backward compatible bug fixes are introduced. A bug fix is defined as an internal change that fixes incorrect behavior.

- Minor version Y (x.Y.z | x > 0) **MUST** be incremented if new, backward compatible functionality is introduced to the public API. It **MUST** be incremented if any public API functionality is marked as deprecated. It **MAY** be incremented if substantial new functionality or improvements are introduced within the private code. It **MAY** include patch level changes. Patch version **MUST** be reset to 0 when minor version is incremented.

- Major version X (X.y.z | X > 0) **MUST** be incremented if any backward incompatible changes are introduced to the public API. It **MAY** also include minor and patch level changes. Patch and minor versions **MUST** be reset to 0 when major version is incremented.

- A pre-release version **MAY** be denoted by appending a hyphen and a series of dot separated identifiers immediately following the patch version. Identifiers **MUST** comprise only ASCII alphanumerics and hyphens [0-9A-Za-z-]. Identifiers **MUST NOT** be empty. Numeric identifiers **MUST NOT** include leading zeroes. Pre-release versions have a lower precedence than the associated normal version. Examples: 1.0.0-alpha, 1.0.0-alpha.1, 1.0.0-0.3.7, 1.0.0-x.7.z.92, 1.0.0-x-y-z.--.

- Build metadata **MAY** be denoted by appending a plus sign and a series of dot separated identifiers immediately following the patch or pre-release version. Identifiers **MUST** comprise only ASCII alphanumerics and hyphens [0-9A-Za-z-]. Identifiers **MUST NOT** be empty. Build metadata **MUST** be ignored when determining version precedence. Thus two versions that differ only in the build metadata, have the same precedence. Examples: 1.0.0-alpha+001, 1.0.0+20130313144700, 1.0.0-beta+exp.sha.5114f85, 1.0.0+21AF26D3----117B344092BD.

### Precedence

- Precedence refers to how versions are compared to each other when ordered.

- Precedence **MUST** be calculated by separating the version into major, minor, patch and pre-release identifiers in that order (Build metadata does not figure into precedence).

- Precedence is determined by the first difference when comparing each of these identifiers from left to right as follows: Major, minor, and patch versions are always compared numerically.
Example: 1.0.0 < 2.0.0 < 2.1.0 < 2.1.1.

- When major, minor, and patch are equal, a pre-release version has lower precedence than a normal version:
Example: 1.0.0-alpha < 1.0.0.

- Precedence for two pre-release versions with the same major, minor, and patch version **MUST** be determined by comparing each dot separated identifier from left to right until a difference is found as follows:
- Identifiers consisting of only digits are compared numerically.
- Identifiers with letters or hyphens are compared lexically in ASCII sort order.
- Numeric identifiers always have lower precedence than non-numeric identifiers.
- A larger set of pre-release fields has a higher precedence than a smaller set, if all of the preceding identifiers are equal.
Example: 1.0.0-alpha < 1.0.0-alpha.1 < 1.0.0-alpha.beta < 1.0.0-beta < 1.0.0-beta.2 < 1.0.0-beta.11 < 1.0.0-rc.1 < 1.0.0.

### Backus–Naur Form Grammar for Valid SemVer Versions

```plaintext
<valid semver> ::= <version core>
| <version core> "-" <pre-release>
| <version core> "+" <build>
| <version core> "-" <pre-release> "+" <build>
<version core> ::= <major> "." <minor> "." <patch>
<major> ::= <numeric identifier>
<minor> ::= <numeric identifier>
<patch> ::= <numeric identifier>
<pre-release> ::= <dot-separated pre-release identifiers>
<dot-separated pre-release identifiers> ::= <pre-release identifier>
| <pre-release identifier> "." <dot-separated pre-release identifiers>
<build> ::= <dot-separated build identifiers>
<dot-separated build identifiers> ::= <build identifier>
| <build identifier> "." <dot-separated build identifiers>
<pre-release identifier> ::= <alphanumeric identifier>
| <numeric identifier>
<build identifier> ::= <alphanumeric identifier>
| <digits>
<alphanumeric identifier> ::= <non-digit>
| <non-digit> <identifier characters>
| <identifier characters> <non-digit>
| <identifier characters> <non-digit> <identifier characters>
<numeric identifier> ::= "0"
| <positive digit>
| <positive digit> <digits>
<identifier characters> ::= <identifier character>
| <identifier character> <identifier characters>
<identifier character> ::= <digit>
| <non-digit>
<non-digit> ::= <letter>
| "-"
<digits> ::= <digit>
| <digit> <digits>
<digit> ::= "0"
| <positive digit>
<positive digit> ::= "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
<letter> ::= "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J"
| "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T"
| "U" | "V" | "W" | "X" | "Y" | "Z" | "a" | "b" | "c" | "d"
| "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n"
| "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x"
```
