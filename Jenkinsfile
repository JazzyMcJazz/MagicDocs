pipeline {
    agent any
    environment {
        PG_PASS = credentials('PG_PASS')
        KC_DB_PASS = credentials('KC_DB_PASS')
        MD_DB_PASS = credentials('MD_DB_PASS')
        KEYCLOAK_ADMIN_PASSWORD = credentials('KEYCLOAK_ADMIN_PASSWORD')
        KEYCLOAK_CLIENT_SECRET = credentials('KEYCLOAK_CLIENT_SECRET')
        SSH_TARGET="lr@159.69.251.4"
    }
    stages {
        stage('Build Images') {
            steps {
                script {
                    sh 'docker build -t pgvector:latest database/'
                    sh 'docker build -t keycloak:latest keycloak/'
                    sh 'docker build -t magicdocs:latest server/'
                }
                script {
                    // save the images to tar files
                    sh 'docker save -o pgvector.tar pgvector:latest'
                    sh 'docker save -o keycloak.tar keycloak:latest'
                    sh 'docker save -o magicdocs.tar magicdocs:latest'
                }
            }
        }
        stage('Deploy to Test Environment') {
            steps {
                script {
                    // commands to deploy using Docker Compose
                    echo 'Todo: Deploy to test environment'
                }
            }
        }
        stage('Run E2E Tests') {
            steps {
                script {
                    // commands to execute Playwright tests
                    // sh 'npm run test:e2e'
                    echo 'Todo: Run E2E tests'
                }
            }
        }
        stage('Cleanup') {
            steps {
                script {
                    // commands to clean up the environment
                    echo 'Todo: Cleanup the environment'
                }
            }
        }
        stage('Deploy to Production') {
            steps {
                withCredentials(bindings: [sshUserPrivateKey(credentialsId: 'DeploymentTargetServer', keyFileVariable: 'SSH_KEY')]) {
                    sh 'scp -i $SSH_KEY pgvector.tar keycloak.tar magicdocs.tar $SSH_TARGET:~/'
                }
                withCredentials(bindings: [sshUserPrivateKey(credentialsId: 'DeploymentTargetServer', keyFileVariable: 'SSH_KEY')]) {
                    sh "ssh -i $SSH_KEY $SSH_TARGET 'docker load -i pgvector.tar && \
                        docker load -i keycloak.tar && \
                        docker load -i magicdocs.tar && \
                        rm pgvector.tar keycloak.tar magicdocs.tar'"
                }
                withCredentials(bindings: [sshUserPrivateKey(credentialsId: 'DeploymentTargetServer', keyFileVariable: 'SSH_KEY')]) {
                    sh "ssh -i $SSH_KEY $SSH_TARGET 'nbot run -f -n magicdocs \
                        -a server \
                            -i magicdocs:latest \
                            -p 3000 \
                            -e RUST_LOG=info \
                            -e RUST_BACKTRACE=0 \
                            -e DATABASE_URL=postgres://magicdocs:$MD_DB_PASS@db:5432/magicdocs \
                            -e KEYCLOAK_INTERNAL_ADDR=http://kc:8080 \
                            -e KEYCLOAK_EXTERNAL_ADDR=https://kc.treeleaf.dev \
                            -e KEYCLOAK_USER=admin \
                            -e KEYCLOAK_PASSWORD=$KEYCLOAK_ADMIN_PASSWORD \
                            -e KEYCLOAK_REALM=magicdocs \
                            -e KEYCLOAK_CLIENT=magicdocs \
                            -e KEYCLOAK_CLIENT_SECRET=$KEYCLOAK_CLIENT_SECRET \
                            -o docs.treeleaf.dev \
                            -m admin@treeleaf.dev \
                            --depends-on db \
                            --depends-on kc \
                        -a kc \
                            -i keycloak:latest \
                            -p 8080 \
                            -e KC_DB=postgres \
                            -e KC_DB_USERNAME=keycloak \
                            -e KC_DB_PASSWORD=$KC_DB_PASS \
                            -e KC_DB_URL_HOST=db \
                            -e KC_DB_URL_PORT=5432 \
                            -e KC_DB_URL_DATABASE=keycloak \
                            -o kc.treeleaf.dev \
                            -m admin@treeleaf.dev \
                            -c \"start --hostname=kc.treeleaf.dev --http-enabled=true --proxy-headers=xforwarded --health-enabled true\" \
                            --depends-on db \
                            --network-alias kc \
                        -a db \
                            -i pgvector:latest \
                            -v magicdocs_db:/var/lib/postgresql/data \
                            --network-alias db'"
                }
            }
        }
    }
}
