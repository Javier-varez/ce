#include <cstdint>
#include <cstdio>
#include <new>
#include <utility>

template <class T, std::size_t N>
class vector {
 public:
  vector() noexcept = default;

  template <typename... Args>
  void emplace_back(Args&&... args) noexcept {
    new (&mData[mSize++ * sizeof(T)]) T(std::forward<Args>(args)...);
  }

 private:
  alignas(T) char mData[sizeof(T) * N];
  std::size_t mSize{0};
};

class Mytype {
 public:
  Mytype(int, const char*) {}

 private:
};

int main() {
  vector<Mytype, 10> i;
  i.emplace_back(1, "");
  printf("Hi there!!\n");
  return 32;
}
