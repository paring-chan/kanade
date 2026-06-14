# API 명세

## 전역 스코프

### ctx
스크립트 전역에서 사용가능한 평가 컨텍스트

#### 필드
- `event`: 이벤트 타입 - push / pull_request / release / cron / manual
- `branch`: 이벤트 발생 브랜치 이름 (비어있을 수 있음)
- `ref`: git ref
- `tag`: git 태그 (태그 푸시 시 포함)
- `args`: 수동 실행 시 지정한 json

#### 함수
- `pipeline("name") -> Pipeline`: 지정한 이름으로 파이프라인 정의 (UI에는 보통 `커밋메시지 (name)` 형식으로 들어감)

## 오브젝트

### Env Map
- 타입: Object Map
- 키: string
- 값: string 또는 #{ secret: "시크릿 키" }

### Pipeline

#### 필드
- `id`: 파이프라인 ID (UUIDv7)
- `name`: 파이프라인 이름

#### 함수
- `job("key", #{ ... }) -> Job`: 설정한 키와 설정으로 비어있는 Job을 생성
- `job("key") -> Job`: 설정한 키와 기본 설정으로 비어있는 Job을 생성

#### Job 설정 오브젝트
모든 키는 필수가 아님(없을 시 기본값 적용)

- `name`: Job 표시 이름
- `image`: 사용할 컨테이너 이미지
- `env`: 사용할 env 맵
- `shell`: 스텝 실행 시 사용할 셸 이름 (기본값: `/bin/bash`)

### Job

#### 필드
- `id`: Job ID (UUIDv7)
- `key`: 파이프라인 내 고유 키
- `name`: Job 이름 (기본값: key와 동일)
- `env`: 사용할 env 맵
- `timeout`: 타임아웃 (분 단위, 기본값: 30분)

#### 함수
- `depend("key") -> Job`: key에 해당하는 job을 의존성 목록에 추가함(자신을 반환)
- `step(#{ ... }) -> Step`: 제공된 설정으로 스텝 생성

#### Step 설정 오브젝트
- `name`: Step 이름(기본값: command의 첫번째 줄)
- `command`: 실행할 명령어
- `env`: 사용할 env 맵

### Step

#### 필드
- `id`: Step ID (UUIDv7)
- `name`: Step 이름
