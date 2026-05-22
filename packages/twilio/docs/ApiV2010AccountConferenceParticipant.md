# ApiV2010AccountConferenceParticipant

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Participant resource. | [optional]
**call_sid** | Option<**String**> | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Participant resource is associated with. | [optional]
**label** | Option<**String**> | The user-specified label of this participant, if one was given when the participant was created. This may be used to fetch, update or delete the participant. | [optional]
**call_sid_to_coach** | Option<**String**> | The SID of the participant who is being `coached`. The participant being coached is the only participant who can hear the participant who is `coaching`. | [optional]
**coaching** | Option<**bool**> | Whether the participant is coaching another call. Can be: `true` or `false`. If not present, defaults to `false` unless `call_sid_to_coach` is defined. If `true`, `call_sid_to_coach` must be defined. | [optional]
**conference_sid** | Option<**String**> | The SID of the conference the participant is in. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**end_conference_on_exit** | Option<**bool**> | Whether the conference ends when the participant leaves. Can be: `true` or `false` and the default is `false`. If `true`, the conference ends and all other participants drop out when the participant leaves. | [optional]
**muted** | Option<**bool**> | Whether the participant is muted. Can be `true` or `false`. | [optional]
**hold** | Option<**bool**> | Whether the participant is on hold. Can be `true` or `false`. | [optional]
**start_conference_on_enter** | Option<**bool**> | Whether the conference starts when the participant joins the conference, if it has not already started. Can be: `true` or `false` and the default is `true`. If `false` and the conference has not started, the participant is muted and hears background music until another participant starts the conference. | [optional]
**status** | Option<[**models::ParticipantEnumStatus**](ParticipantEnumStatus.md)> |  | [optional]
**queue_time** | Option<**String**> | The wait time in milliseconds before participant's call is placed. Only available in the response to a create participant request. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


