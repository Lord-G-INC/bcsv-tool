ifeq ($(OS),Windows_NT)
	TARGET := bcsv-tool.exe
else
	TARGET := bcsv-tool
endif

$(TARGET): main.cc
	clang++ -s -Os $^ -o $@ -static -std=c++17