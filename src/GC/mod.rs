mod VM;

enum ObjectType {
  INT,
  PAIR,
}

struct Pair {
  head: sObject,
  tail: sObject
}

enum InnerUnion {
  INT(i32),
  PAIR(Pair)
}

struct sObject {
  type: ObjectType,
  inner: InnerUnion
}