ifeq ($(OS),Windows_NT)
	TARGET := bcsv-tool.exe
else
	TARGET := bcsv-tool
endif

all: $(TARGET)

$(TARGET): main.cc
	g++ -s -Os $^ -o $@ -static-libstdc++