; TriFlow-AI NSIS Installer Hooks
; 언인스톨 시 바탕화면 바로가기 삭제 및 정리 작업 수행

!macro NSIS_HOOK_PREINSTALL
  ; 설치 전 훅 - 이전 버전 정리
  ; 바탕화면의 기존 바로가기 삭제 (업그레이드 시)
  Delete "$DESKTOP\TriFlow-AI.lnk"
  Delete "$DESKTOP\TriFlow-AI Platform.lnk"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; 설치 후 훅 - 바탕화면 바로가기 생성
  CreateShortCut "$DESKTOP\TriFlow-AI.lnk" "$INSTDIR\TriFlow-AI.exe" "" "$INSTDIR\TriFlow-AI.exe" 0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; 언인스톨 전 훅 - 앱 프로세스 종료
  ; 실행 중인 앱 종료 시도
  nsExec::ExecToLog 'taskkill /F /IM "TriFlow-AI.exe"'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; 언인스톨 후 훅 - 완전 정리

  ; 바탕화면 바로가기 삭제 (모든 가능한 이름)
  Delete "$DESKTOP\TriFlow-AI.lnk"
  Delete "$DESKTOP\TriFlow-AI Platform.lnk"
  Delete "$DESKTOP\TriFlow AI.lnk"

  ; 시작 메뉴 바로가기 삭제
  Delete "$SMPROGRAMS\TriFlow-AI.lnk"
  Delete "$SMPROGRAMS\TriFlow-AI Platform.lnk"
  RMDir "$SMPROGRAMS\TriFlow-AI"

  ; AppData 캐시 정리 (선택적)
  RMDir /r "$APPDATA\TriFlow-AI"
  RMDir /r "$LOCALAPPDATA\TriFlow-AI"

  ; 레지스트리 정리
  DeleteRegKey HKCU "Software\TriFlow-AI"
  DeleteRegKey HKCU "Software\com.triflow.desktop"
!macroend
