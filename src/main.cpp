#include <cstdio>
#include <cstring>

template <unsigned long long N>
struct ConstexprString {
  char name[N];

  constexpr ConstexprString(const char (&str)[N]) {
    for (unsigned long long i = 0; i < N; i++) {
      name[i] = str[i];
    }
  }
};

template <ConstexprString string>
struct ThisIsAStruct {
  static void print() { printf("Value: %s\n", string.name); }
};

int main() {
  ThisIsAStruct<"Noice">::print();
  return 1234;
}
