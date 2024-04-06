# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### 🚀 Features

- Extract provider key in from_str conversion
- Force flush the cache

### 🚜 Refactor

- Represent IP as IpAddr instead of String
- Refactor response struct
- Refactore handling cache
- Ipdate the cache module logic

### 📚 Documentation

- Update examples to new api
- Update comments

### ⚙️ Miscellaneous Tasks

- Add serial test dependency
- Update gitignore
- Adds non_exhaustive to LookupProvider

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


