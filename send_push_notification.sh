curl -X POST https://expo-push-notification-api-rust.vercel.app/api/handler -H "Content-Type: application/json" -d '{
  "title": "cat",
  "body": "meow",
  "expo_push_tokens": "ExponentPushToken[xxxxxxxxxxxxxxxxxxxxxx]"
}'