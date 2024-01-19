### cw-plus-800m

This repository is a stripped down version of cw-plus that only contains a modified cw3 contract. The modifications consist of the following bullet points:

- The instantiation endpoint has been disabled to prevent new contracts to be created from this code
- The execution and migration endpoint execute an handler that burns the contracts USTC balance.

### Use Case

This code is to be used to migrate the 800m multisig contract/wallet  `terra1qyw695vaxj7jl6s4u564c6xkfe59kercg0h88w`
on Terra Classic to a new code version via contract migration proposal. Upon passing of the migration proposal the cw3 wallet in question will effectively

- be blacklisted (as per proposal 11832) and
- the 800m ustc balance will be burned (as per proposal 11660 and 11710)

### Immutability

This code **does not alter** the state of the blockchain. It is going to be compiled to a wasm binary that will be uploaded to the blockchain. A contract migration proposal for the cw3 contract in question is using **stock governance tools** without needing to alter the underlying code of the blockchain. The action performed with this code **cannot** be replicated on any arbitrary regular or multisig wallet and is therefore preserving immutability of the blockchain.

The 800m wallet **happens** to be a cw3 multisig because the multisig owners decided to go for a cw3 wallet instead of a regular one. Only because of this decision we are able to get access to the funds on that wallet and burn them through standard governance tools.
