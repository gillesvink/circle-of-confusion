package com.gillesvink;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;

/**
 * Unit test for Library class.
 */
public class LibraryTest {

    @Test
    public void testGetGreeting() {
        Library library = new Library();
        String expected = "Hello, Alice!";
        assertEquals(expected, library.getGreeting("Alice"));
    }

}
