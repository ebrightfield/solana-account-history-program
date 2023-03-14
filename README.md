## Account History Program

This program does one thing and does it well.
It records historical Solana account state, creating an on-chain record of previous account states.

With this program, you can:
- Create oracle price history feeds, from which common trading statistics can be derived.
- Store an on-chain record of token balance changes.
- Create on-chain logic that gates certain operations by historical presence of certain on-chain data.
- Couple your program's account mutations with update logic, so that the last N state updates are always visible on-chain and easily available.

### How It Works
You create an `AccountHistory` account, which is configured with:
- Some target account whose state is to be copied.
- Some set of data regions to copy.
- A capacity for some number of snapshots.

In either permissionless or permissioned fashion, an `AccountHistory` account can
be updated with the latest snapshot of the target data on the target account,
labeled with the slot number at the time of the data snapshot.

### Testing
```bash
RUST_TEST_NOCAPTURE=1 cargo test --test token
```
The above command will execute a test that creates a new mint/token account,
updates a history account with the token account's balance changes.

### Other Details
- The slot number is prepended to every element of historical data, so the element size will always be >8 bytes long.
- Consumers of account data should specify a type `T:Pod` to read data from an `AccountHistory<T>`. 
- Accounts can have a minimum slot delay between updates.
This enforces a "maximum resolution" to the historical data feed, guaranteeing a minimum age of the data.
- Accounts can be closed, and their lamports retrieved. Admins can add a minimum delay to closing an `AccountHistory`. This allows administrators to relinquish their power of surprise over on-chain consumers of `AccountHistory` data.
