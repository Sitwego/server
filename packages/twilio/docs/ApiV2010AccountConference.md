# ApiV2010AccountConference

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this Conference resource. | [optional]
**date_created** | Option<**String**> | The date and time in UTC that this resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in UTC that this resource was last updated, specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**api_version** | Option<**String**> | The API version used to create this conference. | [optional]
**friendly_name** | Option<**String**> | A string that you assigned to describe this conference room. Maximum length is 128 characters. | [optional]
**region** | Option<**String**> | A string that represents the Twilio Region where the conference audio was mixed. May be `us1`, `us2`, `ie1`,  `de1`, `sg1`, `br1`, `au1`, and `jp1`. Basic conference audio will always be mixed in `us1`. Global Conference audio will be mixed nearest to the majority of participants. | [optional]
**sid** | Option<**String**> | The unique, Twilio-provided string used to identify this Conference resource. | [optional]
**status** | Option<[**models::ConferenceEnumStatus**](ConferenceEnumStatus.md)> |  | [optional]
**uri** | Option<**String**> | The URI of this resource, relative to `https://api.twilio.com`. | [optional]
**subresource_uris** | Option<**serde_json::Value**> | A list of related resources identified by their URIs relative to `https://api.twilio.com`. | [optional]
**reason_conference_ended** | Option<[**models::ConferenceEnumReasonConferenceEnded**](ConferenceEnumReasonConferenceEnded.md)> |  | [optional]
**call_sid_ending_conference** | Option<**String**> | The call SID that caused the conference to end. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


