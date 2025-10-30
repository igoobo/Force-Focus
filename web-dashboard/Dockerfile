# ----------------------------------------------------
# --- Stage 1: Build Stage ---
# 1. 빌드용 베이스 이미지 선택: Node.js 18 버전이 설치된 경량 Alpine 리눅스 이미지를 사용합니다.
#    이 스테이지의 결과물은 'builder'라는 이름으로 참조될 것입니다.
FROM node:18-alpine AS builder

# 2. 작업 디렉토리 설정: 컨테이너 내부의 /app 디렉토리를 작업 공간으로 지정합니다.
WORKDIR /app

# 3. 의존성 설치 (캐싱 최적화):
#    package.json과 package-lock.json (또는 yarn.lock) 파일만 먼저 복사합니다.
#    이 파일들이 변경되지 않으면 이전 빌드의 캐시를 재사용하여 npm install 시간을 절약합니다.
COPY package*.json ./

#    npm 의존성 패키지를 설치합니다.
RUN npm install

# 4. 소스 코드 복사:
#    로컬 디렉토리(web-dashboard/)의 나머지 모든 소스 코드를 컨테이너의 /app 디렉토리로 복사합니다.
COPY . .

# 5. 프로덕션용 앱 빌드:
#    Node.js 환경에서 웹 애플리케이션을 빌드합니다.
#    이 명령어는 package.json에 정의된 "build" 스크립트를 실행합니다 (예: "react-scripts build").
#    결과물은 일반적으로 /app/build 또는 /app/dist 폴더에 생성됩니다.
RUN npm run build

# ----------------------------------------------------
# --- Stage 2: Production Stage ---
# 1. 서빙용 베이스 이미지 선택: 아주 가벼운 Nginx 웹 서버 이미지를 사용합니다.
#    이 스테이지는 'builder' 스테이지의 무거운 Node.js 환경을 포함하지 않습니다.
FROM nginx:stable-alpine

# 2. 빌드된 정적 파일 복사:
#    이전 'builder' 스테이지에서 생성된 빌드 결과물(예: /app/build)을
#    Nginx 웹 서버의 기본 웹 루트 디렉토리(/usr/share/nginx/html)로 복사합니다.
COPY --from=builder /app/build /usr/share/nginx/html

# 3. Nginx 설정 파일 복사:
#    로컬에 준비된 커스텀 Nginx 설정 파일(nginx.conf)을 컨테이너의 Nginx 설정 경로로 복사합니다.
#    이 설정은 주로 SPA(Single Page Application) 라우팅을 위해 필요합니다.
COPY nginx.conf /etc/nginx/conf.d/default.conf

# 4. 포트 노출:
#    컨테이너 내부의 80번 포트(Nginx의 기본 HTTP 포트)가 외부로 노출될 수 있음을 선언합니다.
EXPOSE 80

# 5. Nginx 서버 실행:
#    컨테이너가 시작될 때 Nginx 웹 서버를 실행합니다.
#    "daemon off;"는 Nginx가 포그라운드에서 실행되도록 하여 Docker 컨테이너가 종료되지 않게 합니다.
CMD ["nginx", "-g", "daemon off;"]