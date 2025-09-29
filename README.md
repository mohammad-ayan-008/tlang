````markdown
# TestLang (LLVM + Inkwell Playground)

This project is a toy programming language / compiler backend experiment using **Rust**, **Inkwell** (Rust bindings for LLVM), and **Clang** for linking.  
The goal is to explore LLVM IR generation, function calls, branching, type conversions, and recursive functions such as Fibonacci.

---

## 🔧 Build Instructions

1. **Compile Rust code into LLVM IR & object file**
   ```bash
   cargo run > output.ll
   clang -c output.ll -o output.o
````

2. **Link object file into an executable**

   ```bash
   clang output.o -o myprogram
   ```

3. **Run the executable**

   ```bash
   ./myprogram
   ```

---

## 📦 Features Demonstrated

* **Basic arithmetic** with floats and ints
* **Float to int conversion**
* **Boolean handling** (`true` / `false`)
* **Branching** (unconditional and conditional)
* **Recursive function** (Fibonacci)
* **Printing values** using `printf`

## 📦 Current Language Features

✅ **Numeric types (float, int conversion)**
✅ **Strings and booleans**
✅ **Arithmetic expressions** (`+`, `-`, `*`, `/`)
✅ **Comparison operators** (`<`, `>`, `<=`, `>=`)
✅ **Variables and reassignment**
✅ **`if / else` branching**
✅ **`while` loops**
✅ **Printing values** (`print`)

---

## 📝 Example Program

```c
float a  = (6+7)/2*3;
float b = a+2;
print(b);

string af = "lol";
bool b = true;
print(af);
print(b);

float a  = 6+12;
bool a = 6 < 7;
print(a);

float d = 6;
float c = 9;

if (c >= d) {
   print(c);
} else {
  print("lol");
}

print("-----------fibonacci--------------");
float i = 0;
float a = 0;
float b = 1;

while (i < 10) {
  print(a);
  float next = a + b;
  print(next);
  a = b;
  b = next;
  i = i + 1;
}
```

---

## 🖥️ Example Output

```bash
mohda@ayan:~/testlang$ clang output.o -o myprogram
mohda@ayan:~/testlang$ ./myprogram
21.500000
lol
true
true
9.000000
-----------fibonacci--------------
0.000000
1.000000
1.000000
2.000000
1.000000
3.000000
2.000000
5.000000
3.000000
8.000000
5.000000
13.000000
8.000000
21.000000
13.000000
34.000000
21.000000
55.000000
34.000000
89.000000
```

---

## 🚀 Next Steps

* Add support for **command-line arguments** (`argc`, `argv`)
* Implement more language constructs (`if`, `while`, `for`)
* Extend standard library functions beyond `printf`
* Generate executables directly via LLVM’s target machine

---

## 📝 Notes

This project is purely educational — a sandbox to understand LLVM IR and compiler design in Rust.


