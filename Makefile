
# Build all projects as if they are being published to crates.io, and do so for
# all feature and target combinations.
publish-dry-run-top:
	cargo +stable publish --locked --dry-run --package soroban-wasmi

# Publish publishes the crate to crates.io. The dry-run is a dependency because
# the dry-run target will verify all feature set combinations.
publish-top: publish-dry-run-top
	cargo +stable publish --locked --package soroban-wasmi
