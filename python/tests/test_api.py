import unittest
from permarray import PermutationArray, rank_sjt, unrank_sjt


class ApiTests(unittest.TestCase):
    def test_interior_and_trailing_holes(self):
        a = PermutationArray()
        for value in range(5):
            a.insert(value, value)
        self.assertEqual(a.delete(1), 1)
        self.assertEqual(a.remove_newest(), 4)
        self.assertEqual(a.remove_newest(), 3)
        self.assertEqual(a.remove_newest(), 2)
        self.assertEqual(a.values(), [0])
        self.assertEqual(len(a.slots), 1)
        a.insert(0, 9)
        self.assertEqual(a.values(), [9, 0])

    def test_none_is_a_value(self):
        a = PermutationArray()
        a.insert(0, None)
        a.insert(0, 'x')
        self.assertEqual(a.delete(0), 'x')
        self.assertEqual(a.values(), [None])
        self.assertIsNone(a.remove_newest())
        self.assertEqual(len(a), 0)
        with self.assertRaises(IndexError):
            a.remove_newest()

    def test_invalid_arguments(self):
        with self.assertRaises(ValueError):
            rank_sjt([0, 0])
        with self.assertRaises(ValueError):
            unrank_sjt(6, 3)
        a = PermutationArray()
        with self.assertRaises(IndexError):
            a.insert(-1, 0)
        with self.assertRaises(IndexError):
            a.delete(0)


if __name__ == '__main__':
    unittest.main()
