SAGE ?= sage
SAGE_ENV ?= HOME=$(PWD) SAGE_DOT_SAGE=$(PWD)/.sage
GP_BIN ?= /opt/homebrew/Cellar/pari/2.17.2/bin/gp

.PHONY: prove-security
prove-security:
	$(SAGE_ENV) $(SAGE) -q SAGE/verify_security.sage

.PHONY: prove-security-strong
prove-security-strong:
	$(SAGE_ENV) MOV_MAX_K=1000 TWIST_PROOF=1 TWIST_TRIAL_BOUND=1000000 TWIST_RHO_ATTEMPTS=8 TWIST_RHO_ITERS=50000 $(SAGE) -q SAGE/verify_security.sage

.PHONY: security-notes
security-notes:
	python3 scripts/generate_security_notes.py

.PHONY: vectors
vectors:
	$(SAGE_ENV) $(SAGE) -q SAGE/generate_vectors.sage
	$(SAGE_ENV) $(SAGE) -q SAGE/generate_schnorr_vectors.sage
	$(SAGE_ENV) $(SAGE) -q SAGE/generate_blind_schnorr_vectors.sage

.PHONY: cert-targets
cert-targets:
	$(SAGE_ENV) $(SAGE) -q SAGE/export_cert_targets.sage

.PHONY: prove-l-with-gp
prove-l-with-gp:
	@mkdir -p proved/certs
	$(GP_BIN) -q -f scripts/prove_l_with_gp.gp > proved/certs/l.proof.txt
	@echo "Wrote proved/certs/l.proof.txt"

.PHONY: prove-l-with-sage
prove-l-with-sage:
	@mkdir -p proved/certs
	$(SAGE_ENV) $(SAGE) -python scripts/prove_l_with_sage.py

.PHONY: prove-l
prove-l: prove-l-with-sage

.PHONY: all
all: prove-security-strong vectors security-notes

.PHONY: invalid-corpus
invalid-corpus:
	$(SAGE) -q SAGE/generate_invalid_corpus.sage

.PHONY: ristretto-constants
ristretto-constants:
	$(SAGE_ENV) $(SAGE) -q SAGE/export_ristretto_constants.sage

.PHONY: ristretto-vectors
ristretto-vectors:
	$(SAGE_ENV) $(SAGE) -q SAGE/generate_ristretto_vectors.sage
