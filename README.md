Docker to start RabbitMQ

docker run -d --hostname localhost --name rabbit-server -p 5672:5672 -p 8080:15672 rabbitmq:3-management
