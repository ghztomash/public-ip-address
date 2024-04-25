# Changelog

All notable changes to this project will be documented in this file.

## [0.3.2] - 2024-04-25

### 🚀 Features

- Return error if provider does not support target lookup

### ⚙️ Miscellaneous Tasks

- Remove inline

## [0.3.1] - 2024-04-25

### 🚀 Features

- Add send and sync to Provider

## [0.3.0] - 2024-04-25

### 🚀 Features

- Add ProviderRepsponse trait
- Provide cache file name
- Maybe async feature flag

### 🚜 Refactor

- Provider trait
- Remove make_api_request from Provider trait

### 📚 Documentation

- Update doc tests
- Update documentation for async
- Update documentation for blocking feature
- Add blocking example

### ⚙️ Miscellaneous Tasks

- Remove changelog workflow
- Update dev dependencies
- Run basic example in ci
- Move integration tests to separate module

## [0.2.2] - 2024-04-19

### 🚀 Features

- Encryption feature flag
- Inject logging

### 📚 Documentation

- Update documentation for cache encryption

### ⚙️ Miscellaneous Tasks

- Add publish and changelog workflows

## [0.2.1] - 2024-04-10

### 🚜 Refactor

- Parameter constructor

### 📚 Documentation

- Update documentation

## [0.2.0] - 2024-04-10

### 🚀 Features

- Extract provider key in from_str conversion
- Support IP2Location.io API closes #5 (#7)
- Freeipapi target and key auth
- Adds reverse lookup
- Add  myipcom provider
- Add ipify provider
- Add getjsonip provider
- Cache target lookups as binary tree map (#9)

### 🚜 Refactor

- Represent IP as IpAddr instead of String
- Refactor response struct
- Cache module (#6)
- Pass key and target to API request
- Pass key and target to provider
- Clippy
- Updates api to lookup target

## [0.1.1] - 2024-04-03

### 🚀 Features

- Add iplocate.io provider (#4)
- Add ipleak.net provider
- Add mullvad.net provider
- Add abstractapi.com provider with API key
- Add ipgeolocation.io provider
- Add ipdata.co provider

### 📚 Documentation

- Add git cliff
- Add readme badges
- Add example using provider directtly (#3)
- Update map example

### ⚙️ Miscellaneous Tasks

- Bump reqwest version
- Updates ruty-hook config
- Update pr template
- Update ci/cd workflows
- Typos config


