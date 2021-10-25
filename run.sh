if [ $1 == "server" ];then
    cargo build --bin dorea-$1 && clear && ./target/debug/dorea-$1
elif [ $1 == "client" ];then
    cargo build --bin dorea-$1 && clear && ./target/debug/dorea-$1
else
    ECHO "unknown operation: $1 [ server , client ]"
fi
