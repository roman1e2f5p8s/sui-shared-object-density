# Sui Shared Object Density

The [sui-shared-object-density](https://github.com/roman1e2f5p8s/sui-shared-object-density) 
is a Rust-based project that provides convenient tools to estimate and visualize the density 
of transactions involving shared objects (and other metrics) on the 
[Sui network](https://sui.io/) using the [Sui Rust SDK](https://docs.sui.io/build/rust-sdk). 
Hereafter, the **density** means the ratio of the number of transactions touching shared 
objects to the total number of transactions for a given time interval.

## Table of Contents

- [Background](#background)
    - [Motivation](#motivation)
- [Getting Started](#getting-started)
    - [Install](#install)
    - [Run](#run)
- [Query Usage](#query-usage)
- [Plot Usage](#plot-usage)
- [Examples](#examples)
- [Results](#results)
- [Contribute](#contribute)
- [License](#license)

## Background

Sui is a layer-1 smart contract platform utilizing an object-centric data model: the basic unit 
of storage in Sui is an **object**. The Sui ledger, therefore, stores a collection of 
programmable objects, each with a globally unique ID.

From the ownership point of view, there are two types of Sui objects:
- **Owned objects**: owned by an address and can be used only by transactions signed by that 
owner address at a time. 
- **Shared objects**: no specific owner; anyone can read or write this object. 

Mutable owned objects are **single-writers**, and thus, transactions involving only owned objects 
may bypass the consensus protocols in Sui. Mutable shared objects (**multi-writers**), however, 
require consensus to sequence (order) reads and writes.

### Motivation

Owned objects are the most common case in Sui. Additionally, according to the 
[Sui documentation](https://docs.sui.io/learn/how-sui-works#transactions-on-single-owner-objects), 
many transactions (e.g., asset transfers, NFT minting, smart contract publishing) 
can be realized involving only owned objects. See this 
[list](https://docs.sui.io/learn/single-writer-apps), provided by Sui, of potential 
single-writer real-world applications.

However, 
[Sui documentation](https://docs.sui.io/learn/how-sui-works#transactions-on-shared-objects) 
also claims that many use cases require shared objects that can be manipulated by two or 
more addresses at once (e.g., an auction with open bidding). Therefore, it is not clear how 
often Sui transactions actually touch shared objects, i.e., what the value of the density is. 

Estimating the density in the Sui network would give one an insight into how frequently the use
cases that require shared objects appear on the Sui smart contract platform. The interest in 
knowing the density stems from the fact that transactions with shared object inputs 
require sequencing via the consensus protocol. Therefore, understanding how many apps require
operating with shared objects and what those use cases are is one of the first key steps in 
designing an efficient smart contract architecture.

## Shared Object Analysis Rationale

To analyse a given Sui shared object, different object data options might be used, as specified in
[SuiObjectDataOptions](https://github.com/MystenLabs/sui/blob/2456e2888c15fd843be3370d395f18cafb753563/crates/sui-json-rpc-types/src/sui_object.rs#L326).
With the `show_content` option, a query returns the module name, the object name, the 
`has_public_transfer` field (which indicates where the object has the 
[`store`](https://github.com/MystenLabs/sui/blob/284bf584b46bc3704d0c48cf478923987749a665/sui-execution/latest/sui-adapter/src/programmable_transactions/context.rs#L119)
ability), among others.

The module name and the object name can be used to determine which applications use
Sui shared objects.
The `has_public_transfer` field can be used to determine which shared objects 
are resources. Recall that a [resource in Move](https://move-book.com/resources/what-is-resource.html) 
is a struct that has only `key` and `store` abilities. Therefore, a shared object with 
`has_public_transfer: true` is a shared resource, while shared objects with `has_public_transfer: false`
 are not (they might be Sui system/"protocol" shared objects).

## Getting Started

Before you start, please refer to [Sui Rust SDK documentation](https://docs.sui.io/build/rust-sdk)
and 
[Sui Rust SDK source code](https://github.com/MystenLabs/sui/blob/main/crates/sui-sdk/src/apis.rs)
if you need more information about Sui Rust SDK and available API methods it contains.

### Install

This project assumes `Rust` and `Cargo` are installed, and that there is an available 
internet connection. Please refer to the 
[Rust documentation](https://doc.rust-lang.org/cargo/getting-started/installation.html) 
for the installation instructions.

- Clone the project:
```bash
git clone https://github.com/roman1e2f5p8s/sui-shared-object-density.git
cd sui-shared-object-density
```

- Build the project:
```bash
cargo build --release
```

### Run
Building the project shall create two executable files in:

- UNIX-like
```bash
./target/release/query
./target/release/plot
```

- Windows
```bash
.\target\release\query.exe
.\target\release\plot.exe
```

A more detailed description of how to use these executables is given in the next sections.

## Query Usage

## Plot Usage

## Results
We plot the following characteristics:
- **Total number of transactions** is the total number of Sui transactions per epoch.
- **Number of transactions touching shared objects** is the number (per epoch) of Sui transactions 
that have at least one shared object in their inputs.
- **Density** is the percentage of Sui transactions that touch shared objects, i.e.,
the ratio (multiplied by 100%) of the number of transactions touching shared objects 
(per epoch) to the total number of transactions on Sui (per epoch).
- **Number of shared objects** is the number of shared objects with unique IDs within an epoch.
- **Average contention degree** is the ratio (averaged over intervals within an epoch) of 
the number of transactions touching shared objects to the number of unique shared objects
touched by those transactions within an interval. In other words, this metrics 
tells us how many transactions touch the same shared object on average.
- **Object touchability** is the ratio (averaged over intervals within an epoch) of
the number of shared objects touched by more than one transaction to
the number of unique shared objects. 
