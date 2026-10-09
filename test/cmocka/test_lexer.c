#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include <util/bmem.h>
#include <util/lexer.h>

/* Characterization tests: the expected values are what libobs/util/lexer.c
 * produces today, quirks included. */

static void strref_cmp_test(void **state)
{
	UNUSED_PARAMETER(state);

	struct strref abc, ab, empty, prefix;
	strref_set(&abc, "abc", 3);
	strref_set(&ab, "ab", 2);
	strref_clear(&empty);
	/* a strref is a segment: only the first two chars of the array count */
	strref_set(&prefix, "abcdef", 2);

	assert_int_equal(strref_cmp(&abc, "abc"), 0);
	assert_int_equal(strref_cmp(&abc, "abd"), -1);
	assert_int_equal(strref_cmp(&abc, "abb"), 1);
	/* strref shorter than the C string, and longer than it */
	assert_int_equal(strref_cmp(&ab, "abc"), -1);
	assert_int_equal(strref_cmp(&abc, "ab"), 1);
	assert_int_equal(strref_cmp(&prefix, "ab"), 0);
	assert_int_equal(strref_cmp(&prefix, "abc"), -1);

	/* case sensitive: 'A' (65) < 'a' (97) */
	struct strref upper;
	strref_set(&upper, "ABC", 3);
	assert_int_equal(strref_cmp(&upper, "abc"), -1);
	assert_int_equal(strref_cmp(&abc, "ABC"), 1);

	/* empty strref */
	assert_int_equal(strref_cmp(&empty, ""), 0);
	assert_int_equal(strref_cmp(&empty, NULL), 0);
	assert_int_equal(strref_cmp(&empty, "a"), -1);

	/* non-empty strref against NULL behaves like against "" */
	assert_int_equal(strref_cmp(&abc, NULL), 1);
	/* NULL strref pointer counts as empty */
	assert_int_equal(strref_cmp(NULL, ""), 0);
	assert_int_equal(strref_cmp(NULL, "a"), -1);
}

static void strref_cmpi_test(void **state)
{
	UNUSED_PARAMETER(state);

	struct strref abc, upper, empty;
	strref_set(&abc, "abc", 3);
	strref_set(&upper, "ABC", 3);
	strref_clear(&empty);

	assert_int_equal(strref_cmpi(&abc, "ABC"), 0);
	assert_int_equal(strref_cmpi(&upper, "abc"), 0);
	assert_int_equal(strref_cmpi(&upper, "abd"), -1);
	assert_int_equal(strref_cmpi(&upper, "abb"), 1);
	assert_int_equal(strref_cmpi(&abc, "ab"), 1);
	assert_int_equal(strref_cmpi(&abc, "abcd"), -1);
	assert_int_equal(strref_cmpi(&abc, NULL), 1);

	assert_int_equal(strref_cmpi(&empty, ""), 0);
	assert_int_equal(strref_cmpi(&empty, NULL), 0);
	assert_int_equal(strref_cmpi(&empty, "a"), -1);
}

static void strref_cmp_strref_test(void **state)
{
	UNUSED_PARAMETER(state);

	struct strref abc, abc2, abd, ab, upper, empty, empty2;
	strref_set(&abc, "abc", 3);
	strref_set(&abc2, "abcxyz", 3);
	strref_set(&abd, "abd", 3);
	strref_set(&ab, "ab", 2);
	strref_set(&upper, "ABC", 3);
	strref_clear(&empty);
	strref_clear(&empty2);

	assert_int_equal(strref_cmp_strref(&abc, &abc), 0);
	assert_int_equal(strref_cmp_strref(&abc, &abc2), 0);
	assert_int_equal(strref_cmp_strref(&abc, &abd), -1);
	assert_int_equal(strref_cmp_strref(&abd, &abc), 1);
	assert_int_equal(strref_cmp_strref(&ab, &abc), -1);
	assert_int_equal(strref_cmp_strref(&abc, &ab), 1);
	assert_int_equal(strref_cmp_strref(&upper, &abc), -1);
	assert_int_equal(strref_cmp_strref(&abc, &upper), 1);

	/* empty handling: an empty string sorts before any non-empty one, in
	 * both argument orders (it used to return -1 both ways, #48) */
	assert_int_equal(strref_cmp_strref(&empty, &empty2), 0);
	assert_int_equal(strref_cmp_strref(&empty, &abc), -1);
	assert_int_equal(strref_cmp_strref(&abc, &empty), 1);
}

static void strref_cmpi_strref_test(void **state)
{
	UNUSED_PARAMETER(state);

	struct strref abc, upper, abd, ab, empty, empty2;
	strref_set(&abc, "abc", 3);
	strref_set(&upper, "ABC", 3);
	strref_set(&abd, "ABD", 3);
	strref_set(&ab, "AB", 2);
	strref_clear(&empty);
	strref_clear(&empty2);

	assert_int_equal(strref_cmpi_strref(&abc, &upper), 0);
	assert_int_equal(strref_cmpi_strref(&upper, &abc), 0);
	assert_int_equal(strref_cmpi_strref(&abc, &abd), -1);
	assert_int_equal(strref_cmpi_strref(&abd, &abc), 1);
	assert_int_equal(strref_cmpi_strref(&ab, &abc), -1);
	assert_int_equal(strref_cmpi_strref(&abc, &ab), 1);

	/* same empty ordering as strref_cmp_strref (#48) */
	assert_int_equal(strref_cmpi_strref(&empty, &empty2), 0);
	assert_int_equal(strref_cmpi_strref(&empty, &abc), -1);
	assert_int_equal(strref_cmpi_strref(&abc, &empty), 1);
}

static void valid_int_str_test(void **state)
{
	UNUSED_PARAMETER(state);

	assert_true(valid_int_str("123", 0));
	assert_true(valid_int_str("0", 0));
	assert_true(valid_int_str("-5", 0));
	assert_true(valid_int_str("+7", 0));
	assert_false(valid_int_str("1.5", 0));
	assert_false(valid_int_str("1e3", 0));
	assert_false(valid_int_str("abc", 0));
	assert_false(valid_int_str("12a", 0));
	assert_false(valid_int_str("", 0));
	assert_false(valid_int_str(NULL, 0));
	/* sign only */
	assert_false(valid_int_str("-", 0));

	/* explicit n limits how many chars are examined */
	assert_true(valid_int_str("12a", 2));
	assert_true(valid_int_str("12a", 1));
	assert_false(valid_int_str("12a", 3));
	/* n counts the sign too: "-12x" with n=3 reaches 'x' */
	assert_false(valid_int_str("-12x", 3));
	assert_true(valid_int_str("-12x", 2));
}

static void valid_float_str_test(void **state)
{
	UNUSED_PARAMETER(state);

	assert_true(valid_float_str("123", 0));
	assert_true(valid_float_str("1.5", 0));
	assert_true(valid_float_str("1e3", 0));
	assert_true(valid_float_str("-5", 0));
	assert_true(valid_float_str("+1.5e3", 0));
	assert_true(valid_float_str("1.", 0));
	assert_false(valid_float_str("abc", 0));
	assert_false(valid_float_str("", 0));
	assert_false(valid_float_str(NULL, 0));
	assert_false(valid_float_str(".5", 0));
	assert_false(valid_float_str("1.2.3", 0));
	assert_false(valid_float_str("e3", 0));
	assert_false(valid_float_str("1e3e4", 0));
	assert_false(valid_float_str("1e", 0));
	/* quirk (characterized, not endorsed): a sign right after the exponent
	 * marker is rejected */
	assert_false(valid_float_str("1e-3", 0));
	assert_false(valid_float_str("1e+3", 0));
	/* quirk (characterized, not endorsed): a trailing sign after exponent
	 * digits is accepted */
	assert_true(valid_float_str("1e3-", 0));

	/* explicit n */
	assert_true(valid_float_str("1.5", 1));
	assert_true(valid_float_str("1.x", 2));
	assert_false(valid_float_str("1.x", 3));
}

static void expect_token(struct lexer *lex, enum ignore_whitespace iws, enum base_token_type type, const char *text)
{
	struct base_token t;
	base_token_clear(&t);

	assert_true(lexer_getbasetoken(lex, &t, iws));
	assert_int_equal(t.type, type);
	assert_int_equal(t.text.len, strlen(text));
	assert_int_equal(strref_cmp(&t.text, text), 0);
}

static void expect_end(struct lexer *lex, enum ignore_whitespace iws)
{
	struct base_token t;
	base_token_clear(&t);

	assert_false(lexer_getbasetoken(lex, &t, iws));
}

static void getbasetoken_ignore_ws_test(void **state)
{
	UNUSED_PARAMETER(state);

	long allocs = bnum_allocs();
	struct lexer lex;
	lexer_init(&lex);
	lexer_start(&lex, "abc 12 +\n x");

	expect_token(&lex, IGNORE_WHITESPACE, BASETOKEN_ALPHA, "abc");
	expect_token(&lex, IGNORE_WHITESPACE, BASETOKEN_DIGIT, "12");
	expect_token(&lex, IGNORE_WHITESPACE, BASETOKEN_OTHER, "+");
	expect_token(&lex, IGNORE_WHITESPACE, BASETOKEN_ALPHA, "x");
	expect_end(&lex, IGNORE_WHITESPACE);
	/* stays at the end */
	expect_end(&lex, IGNORE_WHITESPACE);

	/* lexer_reset rewinds to the start */
	lexer_reset(&lex);
	expect_token(&lex, IGNORE_WHITESPACE, BASETOKEN_ALPHA, "abc");

	lexer_free(&lex);
	assert_int_equal(bnum_allocs(), allocs);
}

static void getbasetoken_parse_ws_test(void **state)
{
	UNUSED_PARAMETER(state);

	long allocs = bnum_allocs();
	struct lexer lex;
	lexer_init(&lex);
	lexer_start(&lex, "abc 12 +\n x");

	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_ALPHA, "abc");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_WHITESPACE, " ");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_DIGIT, "12");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_WHITESPACE, " ");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_OTHER, "+");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_WHITESPACE, "\n");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_WHITESPACE, " ");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_ALPHA, "x");
	expect_end(&lex, PARSE_WHITESPACE);

	/* CRLF and LFCR are single two-char whitespace tokens; lone CR is one char */
	lexer_start(&lex, "a\r\nb\n\rc\rd");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_ALPHA, "a");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_WHITESPACE, "\r\n");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_ALPHA, "b");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_WHITESPACE, "\n\r");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_ALPHA, "c");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_WHITESPACE, "\r");
	expect_token(&lex, PARSE_WHITESPACE, BASETOKEN_ALPHA, "d");
	expect_end(&lex, PARSE_WHITESPACE);

	/* whitespace-only text yields nothing when whitespace is ignored */
	lexer_start(&lex, " \t\n ");
	expect_end(&lex, IGNORE_WHITESPACE);

	/* a lexer that was never started has no offset */
	lexer_free(&lex);
	expect_end(&lex, PARSE_WHITESPACE);

	assert_int_equal(bnum_allocs(), allocs);
}

static void getstroffset_test(void **state)
{
	UNUSED_PARAMETER(state);

	long allocs = bnum_allocs();
	struct lexer lex;
	uint32_t row, col;

	lexer_init(&lex);
	lexer_start(&lex, "ab\ncd\r\nef\n\rg");
	const char *t = lex.text;

	row = col = 0;
	lexer_getstroffset(&lex, t, &row, &col);
	assert_int_equal(row, 1);
	assert_int_equal(col, 1);

	lexer_getstroffset(&lex, t + 1, &row, &col);
	assert_int_equal(row, 1);
	assert_int_equal(col, 2);

	/* 'c' directly after "\n" */
	lexer_getstroffset(&lex, t + 3, &row, &col);
	assert_int_equal(row, 2);
	assert_int_equal(col, 1);

	/* 'd' */
	lexer_getstroffset(&lex, t + 4, &row, &col);
	assert_int_equal(row, 2);
	assert_int_equal(col, 2);

	/* the '\r' of the CRLF pair is still on row 2, after "cd" */
	lexer_getstroffset(&lex, t + 5, &row, &col);
	assert_int_equal(row, 2);
	assert_int_equal(col, 3);

	/* 'e' after "\r\n" counts as a single newline */
	lexer_getstroffset(&lex, t + 7, &row, &col);
	assert_int_equal(row, 3);
	assert_int_equal(col, 1);

	/* 'f' */
	lexer_getstroffset(&lex, t + 8, &row, &col);
	assert_int_equal(row, 3);
	assert_int_equal(col, 2);

	/* 'g' after "\n\r" counts as a single newline */
	lexer_getstroffset(&lex, t + 11, &row, &col);
	assert_int_equal(row, 4);
	assert_int_equal(col, 1);

	/* end of text */
	lexer_getstroffset(&lex, t + 12, &row, &col);
	assert_int_equal(row, 4);
	assert_int_equal(col, 2);

	/* NULL pointer leaves the outputs untouched */
	row = 99;
	col = 98;
	lexer_getstroffset(&lex, NULL, &row, &col);
	assert_int_equal(row, 99);
	assert_int_equal(col, 98);

	lexer_free(&lex);
	assert_int_equal(bnum_allocs(), allocs);
}

static void error_data_test(void **state)
{
	UNUSED_PARAMETER(state);

	long allocs = bnum_allocs();
	struct error_data ed;
	error_data_init(&ed);

	/* nothing added yet */
	assert_false(error_data_has_errors(&ed));
	assert_int_equal(error_data_type_count(&ed, LEX_ERROR), 0);
	assert_null(error_data_buildstring(&ed));

	/* adding to a NULL error_data is a no-op */
	error_data_add(NULL, "x.txt", 1, 1, "ignored", LEX_ERROR);

	error_data_add(&ed, "f.txt", 1, 2, "bad", LEX_WARNING);
	/* only warnings so far */
	assert_false(error_data_has_errors(&ed));
	assert_int_equal(error_data_type_count(&ed, LEX_WARNING), 1);

	error_data_add(&ed, "g.txt", 10, 20, "worse", LEX_ERROR);
	assert_true(error_data_has_errors(&ed));
	assert_int_equal(ed.errors.num, 2);
	assert_int_equal(error_data_type_count(&ed, LEX_ERROR), 1);
	assert_int_equal(error_data_type_count(&ed, LEX_WARNING), 1);

	const struct error_item *item = error_data_item(&ed, 1);
	assert_string_equal(item->file, "g.txt");
	assert_string_equal(item->error, "worse");
	assert_int_equal(item->row, 10);
	assert_int_equal(item->column, 20);
	assert_int_equal(item->level, LEX_ERROR);

	char *str = error_data_buildstring(&ed);
	assert_non_null(str);
	assert_string_equal(str, "f.txt (1, 2): bad\ng.txt (10, 20): worse\n");
	bfree(str);

	error_data_free(&ed);
	assert_int_equal(ed.errors.num, 0);
	assert_int_equal(bnum_allocs(), allocs);
}

int main()
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(strref_cmp_test),
		cmocka_unit_test(strref_cmpi_test),
		cmocka_unit_test(strref_cmp_strref_test),
		cmocka_unit_test(strref_cmpi_strref_test),
		cmocka_unit_test(valid_int_str_test),
		cmocka_unit_test(valid_float_str_test),
		cmocka_unit_test(getbasetoken_ignore_ws_test),
		cmocka_unit_test(getbasetoken_parse_ws_test),
		cmocka_unit_test(getstroffset_test),
		cmocka_unit_test(error_data_test),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
