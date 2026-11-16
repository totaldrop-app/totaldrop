The Tamarin lemma and mutant runners (formal/check.sh, formal/mutants.py)
now retry once with +RTS -N1 -M8G -RTS within the same LEMMA_TIMEOUT budget
when tamarin-prover hits an observed nondeterministic "<<loop>>" abort
under its default multi-threaded GHC RTS (upstream root cause unconfirmed);
nothing in the apps, the site or the servers changes (#569).
