# Keep upstream checkouts outside this repository (their licenses differ).
CORPUS_CHECKOUTS ?= ../calculator-upstreams
# Pass CORPUS_LATEST=--latest to fetch upstream default branches instead of pins.
CORPUS_LATEST ?=

.PHONY: checkout-corpus import-corpus
checkout-corpus:
	node scripts/checkout-competitor-suites.mjs --checkouts "$(CORPUS_CHECKOUTS)" $(CORPUS_LATEST)

import-corpus:
	node scripts/import-competitor-corpus.mjs --checkouts "$(CORPUS_CHECKOUTS)"
