pipeline {
    agent any

    environment {
        SSH_TARGET = credentials('SSH_TARGET')
        KEYCLOAK_CLIENT_SECRET = credentials('KEYCLOAK_CLIENT_SECRET')
        OPENAI_API_KEY = credentials('OPENAI_API_KEY')
    }

    stages {
        stage('Build Images') {
            steps {
                script {
                    echo 'Building Docker images'
                    sh 'docker build -t pgvector:latest database/'
                    sh 'docker build -t magicdocs:latest server/'
                    sh 'docker build -t magicdocs_playwright:latest e2e/'
                }
            }
        }
        stage('Run E2E Tests') {
            environment {
                KEYCLOAK_TEST_USER_USERNAME = credentials('KEYCLOAK_TEST_USER_USERNAME')
                KEYCLOAK_TEST_USER_PASSWORD = credentials('KEYCLOAK_TEST_USER_PASSWORD')
                KEYCLOAK_TEST_ADMIN_USERNAME = credentials('KEYCLOAK_TEST_ADMIN_USERNAME')
                KEYCLOAK_TEST_ADMIN_PASSWORD = credentials('KEYCLOAK_TEST_ADMIN_PASSWORD')
                KEYCLOAK_TEST_SUPERADMIN_USERNAME = credentials('KEYCLOAK_TEST_SUPERADMIN_USERNAME')
                KEYCLOAK_TEST_SUPERADMIN_PASSWORD = credentials('KEYCLOAK_TEST_SUPERADMIN_PASSWORD')
            }
            steps {
                // Setup the test environment
                script {
                    echo 'Creating test environment'
                    try {
                        sh 'docker network create magicdocs_test_net'
                    } catch (Exception e) {
                        echo "Failed to create magicdocs_test_net - ${e.getMessage()}"
                    }

                    sh 'docker run -d --rm --name magicdocs_test_db \
                        --network magicdocs_test_net \
                        --network-alias db \
                        -e POSTGRES_PASSWORD=postgres \
                        -e PG_PASS=postgres \
                        -e MD_DB_USER=magicdocs \
                        -e MD_DB_PASS=magicdocs \
                        -v magicdocs_test_db:/var/lib/postgresql/data \
                        --health-cmd=\'pg_isready -U postgres -d magicdocs\' \
                        --health-start-period=10s \
                        --health-start-interval=5s \
                        --health-interval=5m \
                        --health-timeout=10s \
                        --health-retries=3 \
                        pgvector:latest'

                    // Check the health status of the database container
                    def healthy = false
                    def retries = 0
                    while (!healthy && retries < 10) { // Timeout after 30 checks
                        sleep 1 // Wait 10 seconds before each check
                        def status = sh(script: "docker inspect --format='{{.State.Health.Status}}' magicdocs_test_db", returnStdout: true).trim()
                        if (status == "healthy") {
                            healthy = true
                            echo 'Database is ready.'
                        } else {
                            retries++
                            echo "Waiting for database to be healthy... Attempt ${retries}"
                        }
                    }

                    if (!healthy) {
                        error 'Database did not become healthy in time'
                    }

                    sh 'docker run -d --rm --name magicdocs_test_server \
                        --network magicdocs_test_net \
                        --network-alias server \
                        -e RUST_ENV=test \
                        -e DATABASE_URL=postgres://magicdocs:magicdocs@db:5432/magicdocs \
                        -e KEYCLOAK_URL=https://kc.treeleaf.dev \
                        -e KEYCLOAK_USER=$KEYCLOAK_ADMIN_USERNAME \
                        -e KEYCLOAK_PASSWORD=$KEYCLOAK_ADMIN_PASSWORD \
                        -e KEYCLOAK_REALM=magicdocs \
                        -e KEYCLOAK_CLIENT_NAME=magicdocs \
                        -e KEYCLOAK_CLIENT_UUID=987b79aa-e278-4850-8b03-a0f2bf48e05d \
                        -e KEYCLOAK_CLIENT_SECRET=$KEYCLOAK_CLIENT_SECRET \
                        -e OPENAI_API_KEY=$OPENAI_API_KEY \
                        -p 3000:3000 \
                        magicdocs:latest'
                }

                // Run the Playwright tests and save the results
                script {
                    sh 'docker run -d --name magicdocs_test_playwright \
                        --network magicdocs_test_net \
                        -e HOST_URL=http://server:3000 \
                        -e KEYCLOAK_TEST_USER_USERNAME=$KEYCLOAK_TEST_USER_USERNAME \
                        -e KEYCLOAK_TEST_USER_PASSWORD=$KEYCLOAK_TEST_USER_PASSWORD \
                        -e KEYCLOAK_TEST_ADMIN_USERNAME=$KEYCLOAK_TEST_ADMIN_USERNAME \
                        -e KEYCLOAK_TEST_ADMIN_PASSWORD=$KEYCLOAK_TEST_ADMIN_PASSWORD \
                        -e KEYCLOAK_TEST_SUPERADMIN_USERNAME=$KEYCLOAK_TEST_SUPERADMIN_USERNAME \
                        -e KEYCLOAK_TEST_SUPERADMIN_PASSWORD=$KEYCLOAK_TEST_SUPERADMIN_PASSWORD \
                        magicdocs_playwright:latest'

                    def exitCode = sh(script: 'docker wait magicdocs_test_playwright', returnStdout: true).trim().toInteger()
                    echo "Playwright tests exited with code ${exitCode}"

                    sh 'docker cp magicdocs_test_playwright:/app/playwright-report/index.html .'

                    if (exitCode != 0) {
                        error 'Playwright tests failed'
                    }
                }
            }
            post {
                always {
                    echo 'Cleaning up test environment'
                    script {
                        try {
                            archiveArtifacts artifacts: 'index.html',
                                allowEmptyArchive: true
                        } catch (Exception e) {
                            echo "Failed to archive Playwright test results - ${e.getMessage()}"
                        }

                        sh 'docker rm -f magicdocs_test_playwright || true'
                        sh 'docker stop magicdocs_test_server || true'
                        sh 'docker stop magicdocs_test_db || true'
                        sh 'docker network rm magicdocs_test_net || true'
                        sh 'docker rmi magicdocs_playwright:latest || true'
                        sh 'docker volume rm magicdocs_test_db || true'
                    }
                }
            }
        }
        stage('Deploy to Production') {
            environment {
                PG_PASS = credentials('PG_PASS')
                MD_DB_USER = credentials('MD_DB_USER')
                MD_DB_PASS = credentials('MD_DB_PASS')
            }
            steps {
                echo 'Saving Docker images'
                sh 'docker save -o pgvector.tar pgvector:latest'
                sh 'docker save -o magicdocs.tar magicdocs:latest'

                withCredentials(bindings: [sshUserPrivateKey(credentialsId: 'DeploymentTargetServer', keyFileVariable: 'SSH_KEY')]) {
                    echo 'Copying Docker images to production server'
                    sh 'scp -i $SSH_KEY pgvector.tar $SSH_TARGET:~'
                    sh 'scp -i $SSH_KEY magicdocs.tar $SSH_TARGET:~'

                    echo 'Loading Docker images on production server'
                    sh 'ssh -i $SSH_KEY $SSH_TARGET \'docker load -i pgvector.tar && \
                        docker load -i magicdocs.tar\''

                    echo 'Running Docker containers on production server'
                    script {
                        sh "ssh -i $SSH_KEY $SSH_TARGET 'nbot run -f -n magicdocs \
                            -a server \
                                -i magicdocs:latest \
                                -p 3000 \
                                -e MY_LOG=info \
                                -e RUST_BACKTRACE=0 \
                                -e DATABASE_URL=postgres://magicdocs:\\$MD_DB_PASS@db:5432/magicdocs \
                                -e KEYCLOAK_URL=https://kc.treeleaf.dev \
                                -e KEYCLOAK_REALM=magicdocs \
                                -e KEYCLOAK_CLIENT_NAME=magicdocs \
                                -e KEYCLOAK_CLIENT_UUID=987b79aa-e278-4850-8b03-a0f2bf48e05d \
                                -e KEYCLOAK_CLIENT_SECRET=\\$KEYCLOAK_CLIENT_SECRET \
                                -e OPENAI_API_KEY=\\$OPENAI_API_KEY \
                                -o docs.treeleaf.dev \
                                -m admin@treeleaf.dev \
                                --depends-on db \
                            -a db \
                                -i pgvector:latest \
                                -v magicdocs_db:/var/lib/postgresql/data \
                                -e POSTGRES_PASSWORD=\\$PG_PASS \
                                -e MD_DB_USER=\\$MD_DB_USER \
                                -e MD_DB_PASS=\\$MD_DB_PASS \
                                --network-alias db'"
                    }
                }
            }
        }
    }
    post {
        always {
            echo 'Cleaning up local environment'
            script {
                sh 'docker rmi pgvector:latest magicdocs:latest || true'
                sh 'rm pgvector.tar magicdocs.tar || true'
                sh 'docker image prune -f'
                sh 'docker volume prune -f'
            }

            echo 'Cleaning up production server'
            withCredentials(bindings: [sshUserPrivateKey(credentialsId: 'DeploymentTargetServer', keyFileVariable: 'SSH_KEY')]) {
                script {
                    sh 'ssh -i $SSH_KEY $SSH_TARGET \'rm pgvector.tar magicdocs.tar\' || true'
                }
            }
        }
    }
}
