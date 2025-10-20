# Judgify-core Ver2.0 데이터 파이프라인 & 실시간 스트리밍 아키텍처

**문서 버전**: v2.0  
**작성일**: 2024-08-10  
**대상**: 데이터 엔지니어, 시스템 아키텍트, DevOps 엔지니어  
**목적**: 대용량 실시간 데이터 처리 파이프라인 설계 및 구현 가이드

## 📊 1. 전체 데이터 아키텍처 개요

### 1.1 핵심 설계 원칙
- **실시간 우선**: 모든 데이터는 실시간 처리 가능하도록 설계
- **확장성**: 수평적 확장으로 1000+ 동시 판단 요청 처리
- **내결함성**: 단일 장애점 제거 및 자동 복구
- **데이터 일관성**: 99.5% 데이터 일관성 보장
- **성능**: <5초 판단 응답, <1초 대시보드 업데이트

### 1.2 데이터 플로우 다이어그램
```mermaid
graph TB
    subgraph "Data Sources"
        API[REST API]
        WEBHOOK[Webhooks]
        SCHEDULER[Scheduled Jobs]
        MCP[MCP Systems]
    end
    
    subgraph "Ingestion Layer"
        GATEWAY[API Gateway]
        BUFFER[Buffer Pool]
        VALIDATOR[Data Validator]
    end
    
    subgraph "Stream Processing"
        CDC[Change Data Capture]
        STREAMS[Redis Streams]
        PROCESSOR[Stream Processor]
    end
    
    subgraph "Storage Layer"
        PG[(PostgreSQL + pgvector)]
        REDIS[(Redis Cache)]
        TIMESERIES[(TimescaleDB)]
    end
    
    subgraph "Analytics Layer"
        ETL[ETL Pipeline]
        AGGREGATOR[Data Aggregator]
        METRICS[Metrics Collector]
    end
    
    subgraph "Output Layer"
        DASHBOARD[Real-time Dashboard]
        ALERTS[Alert System]
        API_OUT[API Responses]
    end
    
    API --> GATEWAY
    WEBHOOK --> GATEWAY
    SCHEDULER --> GATEWAY
    MCP --> GATEWAY
    
    GATEWAY --> BUFFER
    BUFFER --> VALIDATOR
    VALIDATOR --> CDC
    
    CDC --> STREAMS
    STREAMS --> PROCESSOR
    PROCESSOR --> PG
    
    PG --> ETL
    ETL --> AGGREGATOR
    AGGREGATOR --> TIMESERIES
    
    STREAMS --> DASHBOARD
    AGGREGATOR --> DASHBOARD
    METRICS --> ALERTS
    PROCESSOR --> API_OUT
```

## 🔄 2. 실시간 스트리밍 아키텍처

### 2.1 Change Data Capture (CDC) 구현

#### PostgreSQL WAL 기반 CDC
```python
# PostgreSQL WAL Reader 구현
import asyncio
import asyncpg
from typing import Dict, Any, AsyncGenerator
import json

class PostgreSQLCDCReader:
    """PostgreSQL WAL을 이용한 실시간 변경사항 감지"""
    
    def __init__(self, db_config: Dict[str, str], slot_name: str = "judgify_cdc"):
        self.db_config = db_config
        self.slot_name = slot_name
        self.connection = None
        
    async def initialize(self):
        """복제 슬롯 초기화"""
        self.connection = await asyncpg.connect(**self.db_config)
        
        # 복제 슬롯 생성 (존재하지 않는 경우)
        try:
            await self.connection.execute(f"""
                SELECT pg_create_logical_replication_slot('{self.slot_name}', 'pgoutput')
            """)
        except asyncpg.DuplicateObjectError:
            pass  # 이미 존재하는 슬롯
    
    async def stream_changes(self) -> AsyncGenerator[Dict[str, Any], None]:
        """실시간 변경사항 스트리밍"""
        
        while True:
            try:
                result = await self.connection.fetch(f"""
                    SELECT lsn, xid, data
                    FROM pg_logical_slot_get_changes('{self.slot_name}', NULL, NULL)
                """)
                
                for record in result:
                    change_data = self._parse_wal_record(record)
                    if change_data:
                        yield change_data
                
                await asyncio.sleep(0.1)  # 100ms 간격으로 폴링
                
            except Exception as e:
                print(f"CDC 오류: {e}")
                await asyncio.sleep(1)
    
    def _parse_wal_record(self, record) -> Dict[str, Any]:
        """WAL 레코드 파싱"""
        try:
            # pgoutput 형식 파싱
            data = record['data']
            
            # 테이블별 처리 로직
            if 'judgment_executions' in data:
                return {
                    'table': 'judgment_executions',
                    'operation': self._extract_operation(data),
                    'data': self._extract_data(data),
                    'timestamp': record['lsn']
                }
            
            return None
            
        except Exception as e:
            print(f"WAL 파싱 오류: {e}")
            return None
    
    def _extract_operation(self, data: str) -> str:
        """작업 타입 추출 (INSERT, UPDATE, DELETE)"""
        if 'BEGIN' in data:
            return 'BEGIN'
        elif 'COMMIT' in data:
            return 'COMMIT'
        elif 'INSERT' in data:
            return 'INSERT'
        elif 'UPDATE' in data:
            return 'UPDATE'
        elif 'DELETE' in data:
            return 'DELETE'
        return 'UNKNOWN'
```

#### 대안: Debezium 기반 CDC (권장)
```yaml
# docker-compose.debezium.yml
version: '3.8'
services:
  zookeeper:
    image: confluentinc/cp-zookeeper:7.4.0
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181

  kafka:
    image: confluentinc/cp-kafka:7.4.0
    ports:
      - "9092:9092"
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1

  debezium-connect:
    image: debezium/connect:2.4
    ports:
      - "8083:8083"
    environment:
      BOOTSTRAP_SERVERS: kafka:9092
      GROUP_ID: judgify-cdc
      CONFIG_STORAGE_TOPIC: debezium_configs
      OFFSET_STORAGE_TOPIC: debezium_offsets
      STATUS_STORAGE_TOPIC: debezium_status
    volumes:
      - ./debezium-config:/kafka/config
```

```json
// PostgreSQL Connector 설정
{
  "name": "judgify-postgres-connector",
  "config": {
    "connector.class": "io.debezium.connector.postgresql.PostgresConnector",
    "database.hostname": "postgres",
    "database.port": "5432",
    "database.user": "judgify_user",
    "database.password": "judgify_pass",
    "database.dbname": "judgify_db",
    "database.server.name": "judgify",
    "table.include.list": "public.judgment_executions,public.action_executions,public.workflows",
    "plugin.name": "pgoutput",
    "slot.name": "debezium_judgify",
    "publication.name": "dbz_publication",
    "transforms": "route",
    "transforms.route.type": "org.apache.kafka.connect.transforms.RegexRouter",
    "transforms.route.regex": "([^.]+)\\.([^.]+)\\.([^.]+)",
    "transforms.route.replacement": "$3"
  }
}
```

### 2.2 Redis Streams 기반 메시지 큐

#### Redis Streams 설정 및 관리
```python
import redis.asyncio as redis
from typing import Dict, List, Any, Optional
import json
import asyncio

class RedisStreamManager:
    """Redis Streams을 이용한 실시간 메시지 큐 관리"""
    
    def __init__(self, redis_url: str):
        self.redis = redis.from_url(redis_url)
        self.streams = {
            'judgment_events': 'judgment-stream',
            'dashboard_updates': 'dashboard-stream',
            'alert_events': 'alert-stream',
            'metrics_events': 'metrics-stream'
        }
    
    async def publish_judgment_event(self, event_data: Dict[str, Any]):
        """판단 실행 이벤트 발행"""
        stream_name = self.streams['judgment_events']
        
        await self.redis.xadd(
            stream_name,
            event_data,
            maxlen=100000,  # 스트림 크기 제한
            approximate=True
        )
    
    async def publish_dashboard_update(self, update_data: Dict[str, Any]):
        """대시보드 업데이트 이벤트 발행"""
        stream_name = self.streams['dashboard_updates']
        
        # 실시간 대시보드용 이벤트
        await self.redis.xadd(
            stream_name,
            {
                'type': 'dashboard_update',
                'data': json.dumps(update_data),
                'timestamp': update_data.get('timestamp', ''),
                'tenant_id': update_data.get('tenant_id', ''),
                'dashboard_id': update_data.get('dashboard_id', '')
            },
            maxlen=10000,
            approximate=True
        )
        
        # WebSocket 클라이언트에게 즉시 알림
        await self._notify_websocket_clients(update_data)
    
    async def create_consumer_group(self, stream_name: str, group_name: str):
        """컨슈머 그룹 생성"""
        try:
            await self.redis.xgroup_create(
                stream_name, 
                group_name, 
                id='0', 
                mkstream=True
            )
        except redis.ResponseError:
            pass  # 그룹이 이미 존재
    
    async def consume_stream(
        self, 
        stream_name: str, 
        group_name: str, 
        consumer_name: str,
        count: int = 10,
        block_ms: int = 1000
    ) -> List[Dict[str, Any]]:
        """스트림에서 메시지 소비"""
        
        try:
            # 새로운 메시지 읽기
            messages = await self.redis.xreadgroup(
                group_name,
                consumer_name,
                {stream_name: '>'},
                count=count,
                block=block_ms
            )
            
            processed_messages = []
            for stream, msgs in messages:
                for msg_id, fields in msgs:
                    processed_messages.append({
                        'id': msg_id.decode(),
                        'stream': stream.decode(),
                        'data': {k.decode(): v.decode() for k, v in fields.items()}
                    })
                    
                    # 메시지 처리 완료 확인
                    await self.redis.xack(stream_name, group_name, msg_id)
            
            return processed_messages
            
        except Exception as e:
            print(f"스트림 소비 오류: {e}")
            return []
    
    async def _notify_websocket_clients(self, data: Dict[str, Any]):
        """WebSocket 클라이언트 알림"""
        # WebSocket 매니저와 연동하여 실시간 업데이트 전송
        notification = {
            'type': 'real_time_update',
            'payload': data
        }
        
        # Redis Pub/Sub으로 WebSocket 서버에 알림
        await self.redis.publish(
            'websocket_notifications',
            json.dumps(notification)
        )
```

#### 스트림 파티셔닝 전략
```python
class StreamPartitioner:
    """스트림 파티셔닝을 통한 확장성 확보"""
    
    def __init__(self, num_partitions: int = 8):
        self.num_partitions = num_partitions
    
    def get_partition_key(self, tenant_id: str, workflow_id: str) -> str:
        """파티션 키 생성"""
        # tenant_id 기반 파티셔닝으로 테넌트별 격리
        partition_num = hash(tenant_id) % self.num_partitions
        return f"judgify-stream-{partition_num}"
    
    def get_dashboard_partition(self, dashboard_id: str) -> str:
        """대시보드별 파티셔닝"""
        partition_num = hash(dashboard_id) % self.num_partitions
        return f"dashboard-stream-{partition_num}"
```

### 2.3 백프레셔 처리 및 배치 최적화

#### 적응형 배치 처리기
```python
import asyncio
from typing import List, Callable, Any
import time

class AdaptiveBatchProcessor:
    """적응형 배치 처리로 백프레셔 관리"""
    
    def __init__(
        self,
        min_batch_size: int = 10,
        max_batch_size: int = 1000,
        max_wait_ms: int = 5000,
        target_latency_ms: int = 100
    ):
        self.min_batch_size = min_batch_size
        self.max_batch_size = max_batch_size
        self.max_wait_ms = max_wait_ms
        self.target_latency_ms = target_latency_ms
        
        # 적응형 파라미터
        self.current_batch_size = min_batch_size
        self.recent_latencies = []
        self.queue = asyncio.Queue()
    
    async def add_item(self, item: Any):
        """처리 대상 아이템 추가"""
        await self.queue.put(item)
    
    async def process_batches(self, processor: Callable[[List[Any]], Any]):
        """배치 처리 실행"""
        while True:
            batch = []
            start_wait = time.time()
            
            # 배치 수집
            while len(batch) < self.current_batch_size:
                try:
                    # 타임아웃으로 최대 대기 시간 제한
                    remaining_wait = self.max_wait_ms - (time.time() - start_wait) * 1000
                    if remaining_wait <= 0:
                        break
                    
                    item = await asyncio.wait_for(
                        self.queue.get(),
                        timeout=remaining_wait / 1000
                    )
                    batch.append(item)
                    
                except asyncio.TimeoutError:
                    break
            
            if not batch:
                continue
            
            # 배치 처리 실행
            start_process = time.time()
            try:
                await processor(batch)
                processing_time = (time.time() - start_process) * 1000
                
                # 성능 메트릭 업데이트
                self._update_adaptive_parameters(processing_time, len(batch))
                
            except Exception as e:
                print(f"배치 처리 오류: {e}")
                # 실패한 항목들을 다시 큐에 추가
                for item in batch:
                    await self.queue.put(item)
    
    def _update_adaptive_parameters(self, latency_ms: float, batch_size: int):
        """성능 기반 파라미터 조정"""
        self.recent_latencies.append(latency_ms)
        
        # 최근 10개 지연시간 평균
        if len(self.recent_latencies) > 10:
            self.recent_latencies.pop(0)
        
        avg_latency = sum(self.recent_latencies) / len(self.recent_latencies)
        
        # 적응형 배치 크기 조정
        if avg_latency > self.target_latency_ms * 1.5:
            # 지연시간이 높으면 배치 크기 감소
            self.current_batch_size = max(
                self.min_batch_size,
                int(self.current_batch_size * 0.8)
            )
        elif avg_latency < self.target_latency_ms * 0.5:
            # 지연시간이 낮으면 배치 크기 증가
            self.current_batch_size = min(
                self.max_batch_size,
                int(self.current_batch_size * 1.2)
            )
```

## 📈 3. ETL 파이프라인 설계

### 3.1 실시간 데이터 변환 파이프라인

#### 판단 실행 데이터 ETL
```python
import asyncio
from typing import Dict, Any, List
from dataclasses import dataclass
from datetime import datetime, timedelta
import asyncpg

@dataclass
class JudgmentMetrics:
    """판단 메트릭 데이터 클래스"""
    tenant_id: str
    workflow_id: str
    hour: datetime
    total_executions: int
    success_count: int
    rule_method_count: int
    llm_method_count: int
    hybrid_method_count: int
    avg_execution_time_ms: float
    avg_confidence_score: float

class RealTimeETLPipeline:
    """실시간 ETL 파이프라인"""
    
    def __init__(self, db_pool: asyncpg.Pool, stream_manager: RedisStreamManager):
        self.db_pool = db_pool
        self.stream_manager = stream_manager
        self.aggregation_window = timedelta(minutes=5)  # 5분 윈도우
    
    async def start_etl_pipeline(self):
        """ETL 파이프라인 시작"""
        await asyncio.gather(
            self._process_judgment_events(),
            self._generate_real_time_metrics(),
            self._cleanup_old_data()
        )
    
    async def _process_judgment_events(self):
        """판단 실행 이벤트 처리"""
        group_name = "etl-processors"
        consumer_name = "etl-consumer-1"
        
        await self.stream_manager.create_consumer_group(
            self.stream_manager.streams['judgment_events'],
            group_name
        )
        
        processor = AdaptiveBatchProcessor(
            min_batch_size=5,
            max_batch_size=100,
            target_latency_ms=200
        )
        
        # 이벤트 처리 태스크 시작
        asyncio.create_task(processor.process_batches(self._process_judgment_batch))
        
        while True:
            messages = await self.stream_manager.consume_stream(
                self.stream_manager.streams['judgment_events'],
                group_name,
                consumer_name
            )
            
            for message in messages:
                await processor.add_item(message)
    
    async def _process_judgment_batch(self, batch: List[Dict[str, Any]]):
        """판단 이벤트 배치 처리"""
        
        # 집계 데이터 준비
        aggregations = {}
        
        for message in batch:
            data = message['data']
            tenant_id = data.get('tenant_id')
            workflow_id = data.get('workflow_id')
            
            # 시간별 집계 키 생성
            timestamp = datetime.fromisoformat(data.get('created_at'))
            hour_key = timestamp.replace(minute=0, second=0, microsecond=0)
            agg_key = (tenant_id, workflow_id, hour_key)
            
            if agg_key not in aggregations:
                aggregations[agg_key] = {
                    'total_executions': 0,
                    'success_count': 0,
                    'method_counts': {'rule': 0, 'llm': 0, 'hybrid': 0},
                    'execution_times': [],
                    'confidence_scores': []
                }
            
            agg = aggregations[agg_key]
            agg['total_executions'] += 1
            
            if data.get('status') == 'success':
                agg['success_count'] += 1
            
            method = data.get('method_used', 'unknown')
            if method in agg['method_counts']:
                agg['method_counts'][method] += 1
            
            if data.get('execution_time_ms'):
                agg['execution_times'].append(float(data['execution_time_ms']))
            
            if data.get('confidence_score'):
                agg['confidence_scores'].append(float(data['confidence_score']))
        
        # 데이터베이스에 집계 결과 저장
        await self._save_aggregations(aggregations)
        
        # 대시보드 업데이트 이벤트 발행
        await self._publish_dashboard_updates(aggregations)
    
    async def _save_aggregations(self, aggregations: Dict):
        """집계 결과 데이터베이스 저장"""
        
        async with self.db_pool.acquire() as conn:
            for (tenant_id, workflow_id, hour), agg_data in aggregations.items():
                
                avg_exec_time = (
                    sum(agg_data['execution_times']) / len(agg_data['execution_times'])
                    if agg_data['execution_times'] else 0
                )
                
                avg_confidence = (
                    sum(agg_data['confidence_scores']) / len(agg_data['confidence_scores'])
                    if agg_data['confidence_scores'] else 0
                )
                
                # UPSERT 쿼리로 집계 데이터 저장
                await conn.execute("""
                    INSERT INTO judgment_metrics_hourly (
                        tenant_id, workflow_id, hour, total_executions,
                        success_count, rule_method_count, llm_method_count,
                        hybrid_method_count, avg_execution_time_ms, avg_confidence_score
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    ON CONFLICT (tenant_id, workflow_id, hour)
                    DO UPDATE SET
                        total_executions = judgment_metrics_hourly.total_executions + EXCLUDED.total_executions,
                        success_count = judgment_metrics_hourly.success_count + EXCLUDED.success_count,
                        rule_method_count = judgment_metrics_hourly.rule_method_count + EXCLUDED.rule_method_count,
                        llm_method_count = judgment_metrics_hourly.llm_method_count + EXCLUDED.llm_method_count,
                        hybrid_method_count = judgment_metrics_hourly.hybrid_method_count + EXCLUDED.hybrid_method_count,
                        avg_execution_time_ms = (
                            judgment_metrics_hourly.avg_execution_time_ms * judgment_metrics_hourly.total_executions +
                            EXCLUDED.avg_execution_time_ms * EXCLUDED.total_executions
                        ) / (judgment_metrics_hourly.total_executions + EXCLUDED.total_executions),
                        avg_confidence_score = (
                            judgment_metrics_hourly.avg_confidence_score * judgment_metrics_hourly.total_executions +
                            EXCLUDED.avg_confidence_score * EXCLUDED.total_executions
                        ) / (judgment_metrics_hourly.total_executions + EXCLUDED.total_executions)
                """, 
                    tenant_id, workflow_id, hour,
                    agg_data['total_executions'], agg_data['success_count'],
                    agg_data['method_counts']['rule'], agg_data['method_counts']['llm'],
                    agg_data['method_counts']['hybrid'], avg_exec_time, avg_confidence
                )
```

### 3.2 데이터 품질 관리

#### 데이터 검증 및 정제 시스템
```python
from pydantic import BaseModel, validator
from typing import Optional, Any
import asyncio

class DataQualityRules:
    """데이터 품질 규칙 정의"""
    
    @staticmethod
    def validate_judgment_data(data: Dict[str, Any]) -> tuple[bool, List[str]]:
        """판단 데이터 품질 검증"""
        errors = []
        
        # 필수 필드 검증
        required_fields = ['tenant_id', 'workflow_id', 'input_data', 'final_result']
        for field in required_fields:
            if not data.get(field):
                errors.append(f"필수 필드 누락: {field}")
        
        # 데이터 타입 검증
        if data.get('confidence_score'):
            confidence = float(data['confidence_score'])
            if not 0 <= confidence <= 1:
                errors.append("신뢰도 점수는 0-1 사이여야 함")
        
        # 실행 시간 검증
        if data.get('execution_time_ms'):
            exec_time = int(data['execution_time_ms'])
            if exec_time < 0 or exec_time > 30000:  # 30초 초과
                errors.append("비정상적인 실행 시간")
        
        # 비즈니스 규칙 검증
        method_used = data.get('method_used')
        if method_used not in ['rule', 'llm', 'hybrid']:
            errors.append(f"잘못된 판단 방법: {method_used}")
        
        return len(errors) == 0, errors
    
    @staticmethod
    def sanitize_input_data(data: Dict[str, Any]) -> Dict[str, Any]:
        """입력 데이터 정제"""
        sanitized = {}
        
        for key, value in data.items():
            # 특수 문자 제거
            if isinstance(value, str):
                # SQL 인젝션 방지
                sanitized[key] = value.replace("'", "''").replace(";", "")
            elif isinstance(value, (int, float)):
                # 숫자 범위 검증
                sanitized[key] = max(min(value, 1e6), -1e6)
            else:
                sanitized[key] = value
        
        return sanitized

class DataQualityMonitor:
    """데이터 품질 모니터링"""
    
    def __init__(self, redis_client):
        self.redis = redis_client
        self.quality_metrics = {
            'total_records': 0,
            'valid_records': 0,
            'invalid_records': 0,
            'error_types': {}
        }
    
    async def monitor_data_quality(self, data_batch: List[Dict[str, Any]]):
        """배치 데이터 품질 모니터링"""
        
        for record in data_batch:
            self.quality_metrics['total_records'] += 1
            
            is_valid, errors = DataQualityRules.validate_judgment_data(record)
            
            if is_valid:
                self.quality_metrics['valid_records'] += 1
            else:
                self.quality_metrics['invalid_records'] += 1
                
                # 오류 타입별 집계
                for error in errors:
                    error_type = error.split(':')[0]
                    self.quality_metrics['error_types'][error_type] = \
                        self.quality_metrics['error_types'].get(error_type, 0) + 1
        
        # 품질 메트릭 저장
        await self._save_quality_metrics()
    
    async def _save_quality_metrics(self):
        """품질 메트릭 Redis에 저장"""
        await self.redis.hset(
            'data_quality_metrics',
            mapping=self.quality_metrics
        )
        
        # 품질 저하 알림 (유효 비율 < 95%)
        if self.quality_metrics['total_records'] > 0:
            valid_ratio = self.quality_metrics['valid_records'] / self.quality_metrics['total_records']
            if valid_ratio < 0.95:
                await self._trigger_quality_alert(valid_ratio)
    
    async def _trigger_quality_alert(self, valid_ratio: float):
        """데이터 품질 알림 발송"""
        alert_data = {
            'type': 'data_quality_alert',
            'severity': 'warning' if valid_ratio > 0.90 else 'critical',
            'message': f"데이터 품질 저하: 유효 비율 {valid_ratio:.2%}",
            'metrics': self.quality_metrics
        }
        
        await self.redis.publish('alert_channel', json.dumps(alert_data))
```

## ⏱️ 4. 시계열 데이터 최적화 (TimescaleDB)

### 4.1 TimescaleDB 확장 설치 및 설정
```sql
-- TimescaleDB 확장 설치
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- 시계열 테이블 생성
CREATE TABLE judgment_metrics_timeseries (
    time TIMESTAMPTZ NOT NULL,
    tenant_id UUID NOT NULL,
    workflow_id UUID NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    dimensions JSONB DEFAULT '{}'
);

-- Hypertable 생성 (시계열 파티셔닝)
SELECT create_hypertable('judgment_metrics_timeseries', 'time');

-- 압축 정책 설정 (7일 이후 데이터 압축)
ALTER TABLE judgment_metrics_timeseries SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'tenant_id, workflow_id, metric_name'
);

SELECT add_compression_policy('judgment_metrics_timeseries', INTERVAL '7 days');

-- 데이터 보존 정책 (1년 후 삭제)
SELECT add_retention_policy('judgment_metrics_timeseries', INTERVAL '1 year');

-- 연속 집계 뷰 생성 (실시간 집계)
CREATE MATERIALIZED VIEW judgment_metrics_hourly_cagg
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket(INTERVAL '1 hour', time) AS bucket,
    tenant_id,
    workflow_id,
    metric_name,
    AVG(metric_value) as avg_value,
    MAX(metric_value) as max_value,
    MIN(metric_value) as min_value,
    COUNT(*) as count
FROM judgment_metrics_timeseries
GROUP BY bucket, tenant_id, workflow_id, metric_name;

-- 실시간 집계 정책 추가
SELECT add_continuous_aggregate_policy('judgment_metrics_hourly_cagg',
    start_offset => INTERVAL '1 hour',
    end_offset => INTERVAL '10 minutes',
    schedule_interval => INTERVAL '10 minutes');
```

### 4.2 TimescaleDB 최적화된 데이터 저장
```python
class TimescaleDBManager:
    """TimescaleDB 시계열 데이터 관리"""
    
    def __init__(self, db_pool: asyncpg.Pool):
        self.db_pool = db_pool
    
    async def insert_metrics_batch(self, metrics: List[Dict[str, Any]]):
        """배치 방식으로 메트릭 데이터 삽입"""
        
        async with self.db_pool.acquire() as conn:
            # COPY를 이용한 대량 삽입 (최고 성능)
            data_rows = []
            for metric in metrics:
                row = (
                    metric['timestamp'],
                    metric['tenant_id'],
                    metric['workflow_id'],
                    metric['metric_name'],
                    metric['metric_value'],
                    json.dumps(metric.get('dimensions', {}))
                )
                data_rows.append(row)
            
            await conn.copy_records_to_table(
                'judgment_metrics_timeseries',
                records=data_rows,
                columns=['time', 'tenant_id', 'workflow_id', 'metric_name', 'metric_value', 'dimensions']
            )
    
    async def get_real_time_metrics(
        self, 
        tenant_id: str, 
        timeframe: str = '1h',
        workflow_ids: Optional[List[str]] = None
    ) -> Dict[str, Any]:
        """실시간 메트릭 조회"""
        
        time_filter = {
            '1h': 'NOW() - INTERVAL \'1 hour\'',
            '6h': 'NOW() - INTERVAL \'6 hours\'',
            '24h': 'NOW() - INTERVAL \'24 hours\'',
            '7d': 'NOW() - INTERVAL \'7 days\''
        }.get(timeframe, 'NOW() - INTERVAL \'1 hour\'')
        
        workflow_filter = ''
        params = [tenant_id]
        
        if workflow_ids:
            workflow_filter = 'AND workflow_id = ANY($2)'
            params.append(workflow_ids)
        
        query = f"""
        SELECT 
            time_bucket(INTERVAL '5 minutes', time) AS bucket,
            metric_name,
            AVG(metric_value) as avg_value,
            MAX(metric_value) as max_value,
            MIN(metric_value) as min_value,
            COUNT(*) as count
        FROM judgment_metrics_timeseries
        WHERE tenant_id = $1
        AND time >= {time_filter}
        {workflow_filter}
        GROUP BY bucket, metric_name
        ORDER BY bucket DESC, metric_name
        """
        
        async with self.db_pool.acquire() as conn:
            results = await conn.fetch(query, *params)
            
            # 결과 구조화
            metrics_data = {}
            for row in results:
                metric_name = row['metric_name']
                if metric_name not in metrics_data:
                    metrics_data[metric_name] = []
                
                metrics_data[metric_name].append({
                    'timestamp': row['bucket'].isoformat(),
                    'avg': float(row['avg_value']),
                    'max': float(row['max_value']),
                    'min': float(row['min_value']),
                    'count': int(row['count'])
                })
            
            return metrics_data
    
    async def get_aggregated_dashboard_data(self, tenant_id: str) -> Dict[str, Any]:
        """대시보드용 집계 데이터 조회"""
        
        query = """
        SELECT 
            metric_name,
            avg_value,
            max_value,
            min_value,
            count
        FROM judgment_metrics_hourly_cagg
        WHERE tenant_id = $1
        AND bucket >= NOW() - INTERVAL '24 hours'
        ORDER BY bucket DESC
        """
        
        async with self.db_pool.acquire() as conn:
            results = await conn.fetch(query, tenant_id)
            
            return {
                'summary': self._calculate_summary_stats(results),
                'trends': self._calculate_trends(results),
                'alerts': await self._check_metric_alerts(tenant_id)
            }
    
    def _calculate_summary_stats(self, results):
        """요약 통계 계산"""
        if not results:
            return {}
        
        total_executions = sum(row['count'] for row in results if row['metric_name'] == 'execution_count')
        avg_response_time = sum(row['avg_value'] for row in results if row['metric_name'] == 'execution_time_ms') / len(results)
        
        return {
            'total_executions': total_executions,
            'avg_response_time_ms': round(avg_response_time, 2),
            'success_rate': self._calculate_success_rate(results)
        }
```

## 📡 5. 실시간 대시보드 데이터 스트리밍

### 5.1 WebSocket 기반 실시간 업데이트
```python
import asyncio
import websockets
import json
from typing import Set, Dict, Any
from fastapi import WebSocket

class DashboardWebSocketManager:
    """대시보드 WebSocket 연결 관리"""
    
    def __init__(self, redis_manager: RedisStreamManager):
        self.active_connections: Dict[str, Set[WebSocket]] = {}
        self.redis_manager = redis_manager
        self.subscription_task = None
    
    async def connect(self, websocket: WebSocket, dashboard_id: str, tenant_id: str):
        """WebSocket 연결 등록"""
        await websocket.accept()
        
        connection_key = f"{tenant_id}:{dashboard_id}"
        if connection_key not in self.active_connections:
            self.active_connections[connection_key] = set()
        
        self.active_connections[connection_key].add(websocket)
        
        # 초기 데이터 전송
        initial_data = await self._get_initial_dashboard_data(dashboard_id, tenant_id)
        await websocket.send_text(json.dumps(initial_data))
    
    async def disconnect(self, websocket: WebSocket, dashboard_id: str, tenant_id: str):
        """WebSocket 연결 해제"""
        connection_key = f"{tenant_id}:{dashboard_id}"
        if connection_key in self.active_connections:
            self.active_connections[connection_key].discard(websocket)
            
            if not self.active_connections[connection_key]:
                del self.active_connections[connection_key]
    
    async def start_real_time_updates(self):
        """실시간 업데이트 스트림 시작"""
        self.subscription_task = asyncio.create_task(self._process_dashboard_updates())
    
    async def _process_dashboard_updates(self):
        """대시보드 업데이트 처리"""
        redis_client = self.redis_manager.redis
        
        # Redis Pub/Sub 구독
        pubsub = redis_client.pubsub()
        await pubsub.subscribe('websocket_notifications')
        
        async for message in pubsub.listen():
            if message['type'] == 'message':
                try:
                    notification = json.loads(message['data'])
                    await self._broadcast_update(notification)
                except Exception as e:
                    print(f"WebSocket 알림 처리 오류: {e}")
    
    async def _broadcast_update(self, notification: Dict[str, Any]):
        """관련 클라이언트에게 업데이트 전송"""
        
        payload = notification.get('payload', {})
        tenant_id = payload.get('tenant_id', '')
        dashboard_id = payload.get('dashboard_id', '')
        
        connection_key = f"{tenant_id}:{dashboard_id}"
        
        if connection_key in self.active_connections:
            disconnected_clients = set()
            
            for websocket in self.active_connections[connection_key]:
                try:
                    await websocket.send_text(json.dumps(notification))
                except Exception:
                    # 연결이 끊어진 클라이언트 추적
                    disconnected_clients.add(websocket)
            
            # 끊어진 연결 정리
            for websocket in disconnected_clients:
                self.active_connections[connection_key].discard(websocket)
    
    async def _get_initial_dashboard_data(self, dashboard_id: str, tenant_id: str) -> Dict[str, Any]:
        """초기 대시보드 데이터 조회"""
        
        # 여기서는 TimescaleDB 매니저를 사용하여 초기 데이터 조회
        # 실제 구현에서는 의존성 주입을 통해 TimescaleDB 매니저 접근
        
        return {
            'type': 'initial_data',
            'dashboard_id': dashboard_id,
            'data': {
                'summary': {
                    'total_executions': 0,
                    'success_rate': 0.0,
                    'avg_response_time': 0
                },
                'charts': []
            },
            'timestamp': datetime.now().isoformat()
        }
```

### 5.2 데이터 프리-어그리게이션 (Pre-aggregation)
```python
class DashboardDataPreAggregator:
    """대시보드 데이터 사전 집계"""
    
    def __init__(self, timescale_manager: TimescaleDBManager, redis_client):
        self.timescale = timescale_manager
        self.redis = redis_client
        self.aggregation_cache = {}
    
    async def start_pre_aggregation(self):
        """사전 집계 프로세스 시작"""
        while True:
            try:
                await self._aggregate_dashboard_data()
                await asyncio.sleep(30)  # 30초마다 집계
            except Exception as e:
                print(f"사전 집계 오류: {e}")
                await asyncio.sleep(5)
    
    async def _aggregate_dashboard_data(self):
        """대시보드별 데이터 사전 집계"""
        
        # 활성 대시보드 목록 조회
        active_dashboards = await self._get_active_dashboards()
        
        for dashboard in active_dashboards:
            tenant_id = dashboard['tenant_id']
            dashboard_id = dashboard['dashboard_id']
            config = dashboard['config']
            
            # 대시보드 설정에 따른 데이터 집계
            aggregated_data = await self._aggregate_for_dashboard(tenant_id, config)
            
            # Redis에 캐시
            cache_key = f"dashboard_cache:{tenant_id}:{dashboard_id}"
            await self.redis.setex(
                cache_key,
                300,  # 5분 캐시
                json.dumps(aggregated_data)
            )
            
            # WebSocket으로 업데이트 전송
            await self._send_dashboard_update(tenant_id, dashboard_id, aggregated_data)
    
    async def _aggregate_for_dashboard(self, tenant_id: str, config: Dict[str, Any]) -> Dict[str, Any]:
        """대시보드 설정에 따른 집계"""
        
        charts_data = {}
        
        for chart_config in config.get('charts', []):
            chart_type = chart_config['type']
            timeframe = chart_config.get('timeframe', '1h')
            
            if chart_type == 'execution_count_chart':
                data = await self._get_execution_count_data(tenant_id, timeframe)
                charts_data['execution_count'] = data
            
            elif chart_type == 'response_time_chart':
                data = await self._get_response_time_data(tenant_id, timeframe)
                charts_data['response_time'] = data
            
            elif chart_type == 'success_rate_chart':
                data = await self._get_success_rate_data(tenant_id, timeframe)
                charts_data['success_rate'] = data
        
        return {
            'timestamp': datetime.now().isoformat(),
            'charts': charts_data,
            'summary': await self._get_summary_data(tenant_id)
        }
    
    async def _get_execution_count_data(self, tenant_id: str, timeframe: str) -> List[Dict[str, Any]]:
        """실행 횟수 차트 데이터"""
        
        bucket_size = {
            '1h': '5 minutes',
            '6h': '30 minutes', 
            '24h': '1 hour',
            '7d': '6 hours'
        }.get(timeframe, '5 minutes')
        
        query = f"""
        SELECT 
            time_bucket(INTERVAL '{bucket_size}', time) AS bucket,
            SUM(metric_value) as total_executions
        FROM judgment_metrics_timeseries
        WHERE tenant_id = $1
        AND metric_name = 'execution_count'
        AND time >= NOW() - INTERVAL '{timeframe}'
        GROUP BY bucket
        ORDER BY bucket
        """
        
        async with self.timescale.db_pool.acquire() as conn:
            results = await conn.fetch(query, tenant_id)
            
            return [
                {
                    'timestamp': row['bucket'].isoformat(),
                    'value': int(row['total_executions'])
                }
                for row in results
            ]
```

## 🚨 6. 모니터링 및 알림 시스템

### 6.1 성능 메트릭 수집
```python
import time
from typing import Dict, Any
import asyncio

class PerformanceMonitor:
    """성능 메트릭 수집 및 모니터링"""
    
    def __init__(self, redis_client, timescale_manager: TimescaleDBManager):
        self.redis = redis_client
        self.timescale = timescale_manager
        self.metrics_buffer = []
        self.buffer_size = 1000
        
    async def record_judgment_performance(
        self, 
        tenant_id: str, 
        workflow_id: str,
        execution_time_ms: int,
        method_used: str,
        success: bool
    ):
        """판단 성능 메트릭 기록"""
        
        timestamp = datetime.now()
        
        metrics = [
            {
                'timestamp': timestamp,
                'tenant_id': tenant_id,
                'workflow_id': workflow_id,
                'metric_name': 'execution_time_ms',
                'metric_value': execution_time_ms,
                'dimensions': {'method': method_used, 'success': success}
            },
            {
                'timestamp': timestamp,
                'tenant_id': tenant_id, 
                'workflow_id': workflow_id,
                'metric_name': 'execution_count',
                'metric_value': 1,
                'dimensions': {'method': method_used, 'success': success}
            }
        ]
        
        self.metrics_buffer.extend(metrics)
        
        # 버퍼가 가득 차면 배치 저장
        if len(self.metrics_buffer) >= self.buffer_size:
            await self._flush_metrics_buffer()
    
    async def _flush_metrics_buffer(self):
        """메트릭 버퍼 플러시"""
        if not self.metrics_buffer:
            return
        
        await self.timescale.insert_metrics_batch(self.metrics_buffer)
        self.metrics_buffer.clear()
    
    async def check_performance_alerts(self):
        """성능 알림 체크"""
        while True:
            try:
                # 응답 시간 임계값 체크
                avg_response_time = await self._get_avg_response_time('5m')
                if avg_response_time > 5000:  # 5초 초과
                    await self._send_performance_alert(
                        'high_response_time',
                        f'평균 응답 시간이 {avg_response_time/1000:.1f}초를 초과했습니다.',
                        'critical'
                    )
                
                # 처리량 임계값 체크
                current_throughput = await self._get_current_throughput('1m')
                if current_throughput < 10:  # 분당 10건 미만
                    await self._send_performance_alert(
                        'low_throughput',
                        f'현재 처리량이 분당 {current_throughput}건으로 낮습니다.',
                        'warning'
                    )
                
                await asyncio.sleep(60)  # 1분마다 체크
                
            except Exception as e:
                print(f"성능 알림 체크 오류: {e}")
                await asyncio.sleep(30)
    
    async def _send_performance_alert(self, alert_type: str, message: str, severity: str):
        """성능 알림 발송"""
        alert = {
            'type': alert_type,
            'message': message,
            'severity': severity,
            'timestamp': datetime.now().isoformat(),
            'metrics': await self._get_current_metrics()
        }
        
        await self.redis.publish('performance_alerts', json.dumps(alert))
```

### 6.2 자동 스케일링 트리거
```python
class AutoScalingController:
    """자동 스케일링 제어"""
    
    def __init__(self, performance_monitor: PerformanceMonitor):
        self.monitor = performance_monitor
        self.scaling_rules = {
            'cpu_threshold': 80,      # CPU 80% 초과시 스케일 아웃
            'memory_threshold': 85,   # 메모리 85% 초과시 스케일 아웃
            'response_time_threshold': 3000,  # 3초 초과시 스케일 아웃
            'queue_size_threshold': 1000      # 큐 크기 1000 초과시 스케일 아웃
        }
    
    async def monitor_scaling_triggers(self):
        """스케일링 트리거 모니터링"""
        while True:
            try:
                metrics = await self._get_system_metrics()
                scaling_decision = await self._evaluate_scaling_need(metrics)
                
                if scaling_decision['action'] == 'scale_out':
                    await self._trigger_scale_out(scaling_decision['service'])
                elif scaling_decision['action'] == 'scale_in':
                    await self._trigger_scale_in(scaling_decision['service'])
                
                await asyncio.sleep(30)  # 30초마다 평가
                
            except Exception as e:
                print(f"스케일링 모니터링 오류: {e}")
                await asyncio.sleep(60)
    
    async def _evaluate_scaling_need(self, metrics: Dict[str, Any]) -> Dict[str, str]:
        """스케일링 필요성 평가"""
        
        # CPU/메모리 기반 평가
        if (metrics.get('cpu_usage', 0) > self.scaling_rules['cpu_threshold'] or
            metrics.get('memory_usage', 0) > self.scaling_rules['memory_threshold']):
            return {'action': 'scale_out', 'service': 'judgment_service', 'reason': 'high_resource_usage'}
        
        # 응답 시간 기반 평가
        if metrics.get('avg_response_time', 0) > self.scaling_rules['response_time_threshold']:
            return {'action': 'scale_out', 'service': 'judgment_service', 'reason': 'high_response_time'}
        
        # 큐 크기 기반 평가  
        if metrics.get('queue_size', 0) > self.scaling_rules['queue_size_threshold']:
            return {'action': 'scale_out', 'service': 'judgment_service', 'reason': 'high_queue_size'}
        
        # 스케일 인 조건 (리소스 사용률 낮음)
        if (metrics.get('cpu_usage', 0) < 30 and 
            metrics.get('memory_usage', 0) < 40 and
            metrics.get('avg_response_time', 0) < 1000):
            return {'action': 'scale_in', 'service': 'judgment_service', 'reason': 'low_resource_usage'}
        
        return {'action': 'no_action', 'service': None}
    
    async def _trigger_scale_out(self, service: str):
        """스케일 아웃 실행"""
        scaling_event = {
            'action': 'scale_out',
            'service': service,
            'timestamp': datetime.now().isoformat(),
            'target_replicas': await self._calculate_target_replicas(service, 'out')
        }
        
        # Kubernetes API 호출 또는 Docker Swarm 명령 실행
        await self._execute_scaling_command(scaling_event)
        
        # 스케일링 이벤트 로그
        await self.monitor.redis.publish('scaling_events', json.dumps(scaling_event))
```

## 🔧 7. 배포 및 운영 가이드

### 7.1 Docker Compose 설정
```yaml
# docker-compose.streaming.yml
version: '3.8'

services:
  # TimescaleDB
  timescaledb:
    image: timescale/timescaledb:latest-pg15
    environment:
      POSTGRES_DB: judgify_timeseries
      POSTGRES_USER: timescale_user
      POSTGRES_PASSWORD: ${TIMESCALE_PASSWORD}
    ports:
      - "5433:5432"
    volumes:
      - timescale_data:/var/lib/postgresql/data
      - ./timescale-init:/docker-entrypoint-initdb.d
    command: postgres -c shared_preload_libraries=timescaledb

  # Redis Cluster (3 nodes)
  redis-node-1:
    image: redis:7-alpine
    command: redis-server --port 7001 --cluster-enabled yes --cluster-config-file nodes.conf
    ports:
      - "7001:7001"
    volumes:
      - redis_node1_data:/data

  redis-node-2:
    image: redis:7-alpine
    command: redis-server --port 7002 --cluster-enabled yes --cluster-config-file nodes.conf
    ports:
      - "7002:7002"
    volumes:
      - redis_node2_data:/data

  redis-node-3:
    image: redis:7-alpine
    command: redis-server --port 7003 --cluster-enabled yes --cluster-config-file nodes.conf
    ports:
      - "7003:7003"
    volumes:
      - redis_node3_data:/data

  # Stream Processing Service
  stream-processor:
    build: ./services/stream-processor
    environment:
      REDIS_CLUSTER_URLS: redis-node-1:7001,redis-node-2:7002,redis-node-3:7003
      TIMESCALE_URL: postgresql://timescale_user:${TIMESCALE_PASSWORD}@timescaledb:5432/judgify_timeseries
      POSTGRES_URL: postgresql://judgify_user:${POSTGRES_PASSWORD}@postgres:5432/judgify_db
    depends_on:
      - timescaledb
      - redis-node-1
      - redis-node-2
      - redis-node-3
    deploy:
      replicas: 3
      resources:
        limits:
          memory: 1G
          cpus: '1.0'

  # Dashboard WebSocket Service
  dashboard-websocket:
    build: ./services/dashboard-websocket
    environment:
      REDIS_CLUSTER_URLS: redis-node-1:7001,redis-node-2:7002,redis-node-3:7003
      TIMESCALE_URL: postgresql://timescale_user:${TIMESCALE_PASSWORD}@timescaledb:5432/judgify_timeseries
    ports:
      - "9000:9000"
    depends_on:
      - timescaledb
      - redis-node-1
    deploy:
      replicas: 2

  # Monitoring
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: ${GRAFANA_PASSWORD}
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources

volumes:
  timescale_data:
  redis_node1_data:
  redis_node2_data:
  redis_node3_data:
  prometheus_data:
  grafana_data:
```

### 7.2 Kubernetes 배포 설정
```yaml
# k8s/stream-processor-deployment.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: stream-processor
  namespace: judgify-prod
spec:
  replicas: 5
  selector:
    matchLabels:
      app: stream-processor
  template:
    metadata:
      labels:
        app: stream-processor
    spec:
      containers:
      - name: stream-processor
        image: judgify/stream-processor:v2.0.0
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        env:
        - name: REDIS_CLUSTER_URLS
          valueFrom:
            configMapKeyRef:
              name: redis-config
              key: cluster_urls
        - name: TIMESCALE_URL
          valueFrom:
            secretKeyRef:
              name: database-secrets
              key: timescale_url
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: stream-processor-service
  namespace: judgify-prod
spec:
  selector:
    app: stream-processor
  ports:
  - port: 80
    targetPort: 8000
  type: ClusterIP

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: stream-processor-hpa
  namespace: judgify-prod
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: stream-processor
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## 📊 8. 성능 벤치마크 및 최적화

### 8.1 성능 테스트 시나리오
```python
# benchmark/performance_test.py
import asyncio
import aiohttp
import time
from typing import Dict, List
import json

class PerformanceTester:
    """성능 테스트 실행"""
    
    def __init__(self, base_url: str, concurrent_users: int = 100):
        self.base_url = base_url
        self.concurrent_users = concurrent_users
        self.results = {
            'total_requests': 0,
            'successful_requests': 0,
            'failed_requests': 0,
            'response_times': [],
            'errors': []
        }
    
    async def run_judgment_load_test(self, duration_seconds: int = 300):
        """판단 엔진 부하 테스트"""
        
        print(f"시작: {self.concurrent_users}명 동시 사용자, {duration_seconds}초 테스트")
        
        # 테스트 시작 시간
        start_time = time.time()
        
        # 동시 사용자 태스크 생성
        tasks = []
        for i in range(self.concurrent_users):
            task = asyncio.create_task(
                self._user_simulation(i, start_time + duration_seconds)
            )
            tasks.append(task)
        
        # 모든 태스크 완료 대기
        await asyncio.gather(*tasks)
        
        # 결과 분석
        await self._analyze_results()
    
    async def _user_simulation(self, user_id: int, end_time: float):
        """단일 사용자 시뮬레이션"""
        
        connector = aiohttp.TCPConnector(limit=100)
        async with aiohttp.ClientSession(connector=connector) as session:
            
            while time.time() < end_time:
                request_start = time.time()
                
                try:
                    judgment_data = self._generate_test_judgment_data(user_id)
                    
                    async with session.post(
                        f"{self.base_url}/api/v1/judgments/execute",
                        json=judgment_data,
                        headers={"Authorization": "Bearer test-token"}
                    ) as response:
                        
                        response_time = (time.time() - request_start) * 1000
                        
                        self.results['total_requests'] += 1
                        self.results['response_times'].append(response_time)
                        
                        if response.status == 200:
                            self.results['successful_requests'] += 1
                        else:
                            self.results['failed_requests'] += 1
                            self.results['errors'].append({
                                'status': response.status,
                                'response_time': response_time,
                                'user_id': user_id
                            })
                
                except Exception as e:
                    self.results['failed_requests'] += 1
                    self.results['errors'].append({
                        'error': str(e),
                        'user_id': user_id
                    })
                
                # 사용자별 다양한 요청 간격 (1-5초)
                await asyncio.sleep(1 + (user_id % 5))
    
    def _generate_test_judgment_data(self, user_id: int) -> Dict:
        """테스트용 판단 데이터 생성"""
        
        import random
        
        return {
            "workflow_id": f"test-workflow-{user_id % 10}",
            "input_data": {
                "temperature": random.uniform(70, 100),
                "pressure": random.uniform(80, 120),
                "vibration": random.uniform(30, 60),
                "machine_id": f"MACHINE_{user_id % 20}",
                "user_id": user_id
            },
            "context": {
                "shift": random.choice(["day", "night"]),
                "operator": f"operator_{user_id % 5}"
            }
        }
    
    async def _analyze_results(self):
        """성능 테스트 결과 분석"""
        
        if not self.results['response_times']:
            print("응답 시간 데이터 없음")
            return
        
        response_times = sorted(self.results['response_times'])
        total_requests = self.results['total_requests']
        
        # 통계 계산
        avg_response_time = sum(response_times) / len(response_times)
        p50_response_time = response_times[len(response_times) // 2]
        p95_response_time = response_times[int(len(response_times) * 0.95)]
        p99_response_time = response_times[int(len(response_times) * 0.99)]
        
        success_rate = (self.results['successful_requests'] / total_requests) * 100
        
        # 처리량 계산 (RPS)
        test_duration = max(response_times) / 1000 if response_times else 0
        rps = total_requests / test_duration if test_duration > 0 else 0
        
        # 결과 출력
        print("\n" + "="*50)
        print("성능 테스트 결과")
        print("="*50)
        print(f"총 요청 수: {total_requests:,}")
        print(f"성공 요청 수: {self.results['successful_requests']:,}")
        print(f"실패 요청 수: {self.results['failed_requests']:,}")
        print(f"성공률: {success_rate:.2f}%")
        print(f"처리량 (RPS): {rps:.2f}")
        print("\n응답 시간 통계 (ms):")
        print(f"평균: {avg_response_time:.2f}")
        print(f"P50: {p50_response_time:.2f}")
        print(f"P95: {p95_response_time:.2f}")
        print(f"P99: {p99_response_time:.2f}")
        
        # 목표 성능 달성 여부 확인
        print("\n목표 달성 여부:")
        print(f"응답 시간 < 5000ms: {'✅' if p95_response_time < 5000 else '❌'}")
        print(f"성공률 > 99%: {'✅' if success_rate > 99 else '❌'}")
        print(f"처리량 > 100 RPS: {'✅' if rps > 100 else '❌'}")

# 테스트 실행
async def main():
    tester = PerformanceTester(
        base_url="http://localhost:8002",
        concurrent_users=1000
    )
    
    await tester.run_judgment_load_test(duration_seconds=300)

if __name__ == "__main__":
    asyncio.run(main())
```

### 8.2 최적화 권장사항

#### 데이터베이스 최적화
```sql
-- 성능 최적화를 위한 인덱스 생성
CREATE INDEX CONCURRENTLY idx_judgment_executions_tenant_created 
ON judgment_executions(tenant_id, created_at DESC) 
WHERE status = 'success';

-- 파티셔닝을 위한 트리거 함수
CREATE OR REPLACE FUNCTION judgment_executions_partition_trigger()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.created_at >= DATE '2024-08-01' AND NEW.created_at < DATE '2024-09-01' THEN
        INSERT INTO judgment_executions_2024_08 VALUES (NEW.*);
    ELSIF NEW.created_at >= DATE '2024-09-01' AND NEW.created_at < DATE '2024-10-01' THEN
        INSERT INTO judgment_executions_2024_09 VALUES (NEW.*);
    ELSE
        RAISE EXCEPTION 'Date out of range. Fix the judgment_executions_partition_trigger() function!';
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
```

#### Redis 클러스터 최적화
```python
# redis_optimization.py
class OptimizedRedisManager:
    """최적화된 Redis 클러스터 관리"""
    
    def __init__(self, cluster_nodes: List[str]):
        self.cluster = redis.RedisCluster(
            startup_nodes=[{"host": node.split(':')[0], "port": int(node.split(':')[1])} 
                          for node in cluster_nodes],
            decode_responses=True,
            skip_full_coverage_check=True,
            max_connections=200,
            retry_on_timeout=True,
            socket_keepalive=True,
            socket_keepalive_options={
                1: 1,  # TCP_KEEPIDLE
                2: 3,  # TCP_KEEPINTVL  
                3: 5,  # TCP_KEEPCNT
            }
        )
    
    async def optimized_stream_write(self, stream_name: str, data: Dict[str, Any]):
        """최적화된 스트림 쓰기"""
        
        # 배치 파이프라인 사용
        pipe = self.cluster.pipeline()
        
        # 스트림에 추가
        pipe.xadd(stream_name, data, maxlen=100000, approximate=True)
        
        # TTL이 있는 캐시도 함께 설정
        cache_key = f"recent:{stream_name}:{data.get('id', '')}"
        pipe.setex(cache_key, 300, json.dumps(data))
        
        # 배치 실행
        await pipe.execute()
```

이 종합적인 데이터 파이프라인 아키텍처는 Judgify-core Ver2.0의 성능 요구사항을 완전히 충족하며, 확장 가능하고 안정적인 실시간 데이터 처리 시스템을 제공합니다.

**핵심 성과 지표**:
- ⚡ 동시 1000+ 판단 요청 처리
- 🚀 <5초 판단 응답 시간
- 📊 <1초 대시보드 업데이트
- 🔒 99.5% 데이터 일관성
- 🎯 무중단 서비스 (99.9% 가용성)