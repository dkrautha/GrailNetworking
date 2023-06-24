package xbf_kt

class Primitive {
  enum class Discriminants {
    BOOLEAN,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    I8,
    I16,
    I32,
    I64,
    I128,
    I256,
    F32,
    F64,
    STRING;

		companion object {
			fun getNumberOfPrimitives() = values().size;
      fun getDiscriminant(ordinal: Int) = values()[ordinal]; 
		}
  }


}
