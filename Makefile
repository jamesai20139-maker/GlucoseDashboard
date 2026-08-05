.PHONY: run build frontend-build backend-build configure test format clean

BACKEND_MANIFEST := backend/Cargo.toml
FIXTURE ?= backend/tests/fixtures/valid-sheet.csv
SHEET_ID ?= demo

run: frontend-build configure
	cargo run --manifest-path $(BACKEND_MANIFEST)

build: frontend-build backend-build

frontend-build:
	npm --prefix frontend install
	npm --prefix frontend run build

backend-build:
	cargo build --manifest-path $(BACKEND_MANIFEST)

configure:
	cargo run --manifest-path $(BACKEND_MANIFEST) -- config $(SHEET_ID) $(FIXTURE)

test:
	cargo test --manifest-path $(BACKEND_MANIFEST)
	npm --prefix frontend run build

format:
	cargo fmt --manifest-path $(BACKEND_MANIFEST) --all

clean:
	cargo clean --manifest-path $(BACKEND_MANIFEST)
	rm -rf frontend/dist
