RUSTC?=rustc.exe
LIBDIR?=.
SRC=$(wildcard *.rs) $(wildcard ll/*.rs)
MAIN=shinobu.rs
SHINOBU=$(patsubst %.rs,%.exe,$(MAIN))

.PHONY: all
all: $(SHINOBU)

$(SHINOBU): $(SRC)
	$(RUSTC) $(MAIN) -L $(LIBDIR) -o $@

.PHONY: clean
clean:
	rm -f $(SHINOBU)
