
default:
	mkdir -p target/cpp
	g++ -o target/cpp/sha cpp/sha.c
	./target/cpp/sha