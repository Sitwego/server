# \Api20100401FeedbackApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_message_feedback**](Api20100401FeedbackApi.md#create_message_feedback) | **POST** /2010-04-01/Accounts/{AccountSid}/Messages/{MessageSid}/Feedback.json | Create Message Feedback to confirm a tracked user action was performed by the recipient of the associated Message



## create_message_feedback

> models::ApiV2010AccountMessageMessageFeedback create_message_feedback(account_sid, message_sid, outcome)
Create Message Feedback to confirm a tracked user action was performed by the recipient of the associated Message

Create Message Feedback to confirm a tracked user action was performed by the recipient of the associated Message

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) associated with the Message resource for which to create MessageFeedback. | [required] |
**message_sid** | **String** | The SID of the Message resource for which to create MessageFeedback. | [required] |
**outcome** | Option<[**models::MessageFeedbackEnumOutcome**](MessageFeedbackEnumOutcome.md)> |  |  |

### Return type

[**models::ApiV2010AccountMessageMessageFeedback**](api.v2010.account.message.message_feedback.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

