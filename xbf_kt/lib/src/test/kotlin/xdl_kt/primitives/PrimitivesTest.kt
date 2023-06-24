package xbf_kt

import kotlin.test.Test
import kotlin.test.assertFails
import kotlin.test.assertTrue

class PrimitivesTest {
  @Test
  fun testCorrectNumberOfDiscriminants() {
    val classUnderTest = Primitive.Discriminants

    assertTrue(
        (classUnderTest.getNumberOfPrimitives() == 16),
        "Number of Discriminants should return 16 (as per XDL spec)"
    )
  }

  @Test
  fun testDiscriminantValues() {
    val classUnderTest = Primitive.Discriminants

    val testDiscriminant = { ordinal: Int, discriminantString: String ->
      assertTrue(
          (classUnderTest.getDiscriminant(ordinal).toString() == discriminantString),
          "Test that Discriminants are in correct order (${ordinal}, ${discriminantString})"
      )
    }

    testDiscriminant.invoke(0, "BOOLEAN")
    testDiscriminant.invoke(1, "U8")
    testDiscriminant.invoke(2, "U16")
    testDiscriminant.invoke(3, "U32")
    testDiscriminant.invoke(4, "U64")
    testDiscriminant.invoke(5, "U128")
    testDiscriminant.invoke(6, "U256")
    testDiscriminant.invoke(7, "I8")
    testDiscriminant.invoke(8, "I16")
    testDiscriminant.invoke(9, "I32")
    testDiscriminant.invoke(10, "I64")
    testDiscriminant.invoke(11, "I128")
    testDiscriminant.invoke(12, "I256")
    testDiscriminant.invoke(13, "F32")
    testDiscriminant.invoke(14, "F64")
    testDiscriminant.invoke(15, "STRING")

    val testDiscriminantFail = { ordinal: Int ->
      assertFails({ classUnderTest.getDiscriminant(ordinal) })
    }

    testDiscriminantFail.invoke(-1)
    testDiscriminantFail.invoke(16)
  }
}
