parent(john, mary).
parent(john, david).
parent(mary, alice).
parent(david, bob).

ancestor(X, Y) :- parent(X, Y).
ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y).
