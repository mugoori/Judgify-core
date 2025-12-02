/**
 * 공통 Mock 데이터 유틸리티 및 데이터 풀
 */

// 담당자 이름 풀
export const KOREAN_NAMES = [
  '김민수', '이영희', '박철수', '최지영', '정대현',
  '한수진', '윤성호', '강미라', '조현우', '신예진',
  '오승현', '임지훈', '서영수', '배소연', '황정민',
];

// 역할 풀
export const ROLES = {
  QC: 'QC팀장',
  PRODUCTION: '생산팀장',
  MAINTENANCE: '설비팀장',
  WAREHOUSE: '창고팀장',
  PURCHASE: '구매팀장',
  MANAGER: '공장장',
};

// 원료 풀 - 유가공 원료
export const MATERIALS = [
  // 유제품 원료
  '원유 (1등급)', '원유 (2등급)', '탈지분유', '전지분유',
  '농축유', '탈지농축유', '유청분말 (WPC80)', '유청분말 (WPI90)',

  // 유지류
  '무염버터', '가염버터', '생크림', '휘핑크림 베이스',
  '크림치즈 베이스', '버터오일', '유지방',

  // 발효/배양
  'L.불가리쿠스 유산균', 'S.써모필러스 유산균', 'L.카제이 유산균',
  '비피더스균 배양액', 'R-1 유산균', 'LGG 유산균',

  // 당류
  '정제당', '과당', '올리고당', '스테비아', '알룰로스',
  '물엿', '액상과당', '벌꿀',

  // 첨가물
  '바닐라향', '딸기향', '초코향', '바나나향',
  '안정제 (펙틴)', '안정제 (카라기난)', '유화제',

  // 과일/기타
  '딸기퓨레', '블루베리퓨레', '망고퓨레', '복숭아농축액',
  '코코아파우더', '커피원액', '녹차분말',
];

// 제품 풀 - 유음료 제품만 (음료 공정)
export const PRODUCTS = [
  // 백색시유 (흰우유)
  '퓨어밀크 1L', '퓨어밀크 500ml', '퓨어밀크 200ml',
  '저지방 퓨어밀크 1L', '저지방 퓨어밀크 500ml',
  '무지방 퓨어밀크 1L', '유기농 목장우유 900ml',
  '프리미엄 목장우유 1L', '고칼슘 우유 900ml',

  // 가공유 (맛우유)
  '초코몽 200ml', '초코몽 500ml', '딸기몽 200ml',
  '바나나몽 200ml', '커피앤밀크 300ml', '커피앤밀크 500ml',
  '달콤바닐라 우유 200ml', '흑임자 우유 200ml',

  // 발효유 (마시는 타입)
  '프로바이오 드링크 150ml', '프로바이오 드링크 80ml',
  '장건강 발효유 65ml', '장건강 발효유 130ml',
  '쾌변 발효유 130ml', '면역 플러스 발효유 100ml',
  '마시는 요거트 딸기 200ml', '마시는 요거트 블루베리 200ml',
  '마시는 요거트 플레인 200ml', '비피더스 음료 150ml',

  // UHT 멸균유
  '멸균우유 1L', '멸균우유 200ml', '멸균 저지방우유 1L',
  '멸균 초코우유 200ml', '멸균 딸기우유 200ml',

  // 두유/대체유
  '검은콩 두유 190ml', '달콤한 두유 190ml', '무가당 두유 190ml',
  '호두 아몬드 두유 190ml', '오트밀크 950ml', '아몬드밀크 950ml',
  '귀리음료 200ml', '코코넛밀크 200ml',
];

// 공급업체 풀
export const SUPPLIERS = [
  '한국유업', '삼양식품', 'CJ제일제당', '동원F&B',
  '매일유업', '남양유업', '빙그레', '롯데푸드',
];

// 설비 풀 - 유음료 공정 설비
export const EQUIPMENT = [
  // 원유 수입/저장
  '원유저장탱크 T-101', '원유저장탱크 T-102', '원유저장탱크 T-103',
  '원유분리기 SP-01', '청정기 CL-01', '표준화탱크 ST-01',

  // 균질/살균 (CCP)
  '균질기 HG-01', '균질기 HG-02',
  'HTST 살균기 PS-01', 'HTST 살균기 PS-02',
  'UHT 멸균기 UHT-01', 'UHT 멸균기 UHT-02',

  // 발효 (발효유 전용)
  '발효탱크 FT-01', '발효탱크 FT-02', '발효탱크 FT-03',
  '배양탱크 CT-01', '접종장치 IN-01',

  // 배합/혼합 (가공유/두유)
  '배합탱크 MX-01', '배합탱크 MX-02', '용해탱크 DS-01',
  '교반기 AG-01', '인라인믹서 IM-01',

  // 냉각 (CCP)
  '냉각탱크 CL-01', '냉각탱크 CL-02', '판형냉각기 PH-01',

  // 충진/포장
  '우유충진기 FL-01', '우유충진기 FL-02', '우유충진기 FL-03',
  '페트충진기 PT-01', '페트충진기 PT-02',
  '파우치충진기 PF-01', '캡핑기 CP-01', '캡핑기 CP-02',

  // 검사/품질 (CCP)
  '금속검출기 MD-01', '금속검출기 MD-02', 'X-Ray검사기 XR-01',
  '중량선별기 CW-01', '라벨검사기 LI-01',

  // 포장/물류
  '자동포장기 AP-01', '밴딩기 BD-01', '파렛타이저 PL-01',

  // 세정(CIP)
  'CIP 유닛 CIP-01', 'CIP 유닛 CIP-02',
];

// 생산라인 풀 - 유음료 생산라인
export const PRODUCTION_LINES = [
  // 백색시유 라인
  '시유 1라인', '시유 2라인', '저지방 시유 라인',

  // 발효유 라인
  '발효유 A라인', '발효유 B라인', '발효유 C라인',

  // 가공유 라인
  '가공유 1라인', '가공유 2라인',

  // UHT 멸균유 라인
  'UHT 1라인', 'UHT 2라인',

  // 두유/대체유 라인
  '두유 라인', '대체유 라인',
];

// 검사항목 풀
export const INSPECTION_ITEMS = {
  material: ['수분함량', '지방함량', '단백질', '산도', '세균수', '이물질'],
  ccp: ['온도', '시간', '압력', '민감도', 'pH'],
  product: ['맛', '색상', '점도', '당도', '염도', '산도'],
};

// 유틸리티 함수들
export function randomFrom<T>(arr: T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}

export function randomBetween(min: number, max: number, decimals = 1): number {
  const value = Math.random() * (max - min) + min;
  return Number(value.toFixed(decimals));
}

export function randomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

export function generateTimestamp(offsetMinutes = 0): string {
  const now = new Date();
  now.setMinutes(now.getMinutes() - offsetMinutes);
  return now.toISOString().replace('T', ' ').substring(0, 19);
}

export function generateLotNumber(): string {
  const today = new Date();
  const year = today.getFullYear();
  const month = String(today.getMonth() + 1).padStart(2, '0');
  const day = String(today.getDate()).padStart(2, '0');
  const seq = String(randomInt(1, 999)).padStart(3, '0');
  return `LOT-${year}-${month}${day}-${seq}`;
}

export function generateBatchNumber(): string {
  const today = new Date();
  const year = today.getFullYear();
  const month = String(today.getMonth() + 1).padStart(2, '0');
  const day = String(today.getDate()).padStart(2, '0');
  const seq = String(randomInt(1, 99)).padStart(2, '0');
  return `B${year}${month}${day}-${seq}`;
}

// 담당자 정보 생성
export function generateApprover(roleKey: keyof typeof ROLES): { name: string; role: string } {
  return {
    name: randomFrom(KOREAN_NAMES),
    role: ROLES[roleKey],
  };
}

// 수량 생성 (단위 포함)
export function generateQuantity(unit: string, min: number, max: number): string {
  return `${randomInt(min, max)}${unit}`;
}
