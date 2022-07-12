#include <sys/types.h>
#include <sys/socket.h>
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <unistd.h>
#include <netinet/in.h>

#define SERVPORT 4444
#define BACKLOG 10
#define MAXDATASIZE 15

int main() {
    struct sockaddr_in server_sockaddr;//声明服务器socket存储结构
    int sin_size,recvbytes;
    int sockfd,client_fd;//socket描述符
    char buf[MAXDATASIZE];//传输的数据

    //1.建立socket
    //AF_INET 表示IPV4
    //SOCK_STREAM 表示TCP
    if((sockfd = socket(AF_INET,SOCK_STREAM,0)) < 0) {
        perror("Socket");
        exit(1);
    }

    printf("Socket successful!,sockfd=%d\n",sockfd);

    //以sockaddt_in结构体填充socket信息
    server_sockaddr.sin_family 		= AF_INET;//IPv4
    server_sockaddr.sin_port 		= htons(SERVPORT);//端口
    server_sockaddr.sin_addr.s_addr 	= INADDR_ANY;//本主机的任意IP都可以使用
    bzero(&(server_sockaddr.sin_zero),8);//保留的8字节置零

    //2.绑定 fd与 端口和地址
    if((bind(sockfd,(struct sockaddr *)&server_sockaddr,sizeof(struct sockaddr))) < 0) {
        perror("bind");
        exit(-1);
    }

    printf("bind successful !\n");

    //3.监听
    if(listen(sockfd,BACKLOG) < 0) {
        perror("listen");
        exit(1);
    }

    printf("listening ... \n");

    while(1){
	//4.接收请求,函数在有客户端连接时返回一个客户端socket fd,否则则阻塞
	//优化：这里同样可以使用select,以及poll来实现异步通信
	if((client_fd = accept(sockfd,NULL,&sin_size)) == -1) {
		perror("accept");
		exit(1);
	}

	printf("accept success! client_fd:%d\n",client_fd);

	//5.接收数据
        //注意：这里传入的fd，不是建立的socket fd，而是accept返回的连接客户端 socket fd
	if((recvbytes = recv(client_fd,buf,MAXDATASIZE,0)) == -1) {
		perror("recv");
		exit(1);
	}

	printf("received data : %s\n",buf);
    }

    //6.关闭
    close(sockfd);

}